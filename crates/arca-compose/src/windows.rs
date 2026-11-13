use std::mem::ManuallyDrop;
use windows::Win32::Graphics::{
    Direct3D11::{
        D3D11_BIND_RENDER_TARGET, D3D11_TEX2D_VPIV, D3D11_TEX2D_VPOV, D3D11_TEXTURE2D_DESC,
        D3D11_USAGE_DEFAULT, D3D11_VIDEO_FRAME_FORMAT_PROGRESSIVE,
        D3D11_VIDEO_PROCESSOR_CONTENT_DESC, D3D11_VIDEO_PROCESSOR_FORMAT_SUPPORT_INPUT,
        D3D11_VIDEO_PROCESSOR_FORMAT_SUPPORT_OUTPUT, D3D11_VIDEO_PROCESSOR_INPUT_VIEW_DESC,
        D3D11_VIDEO_PROCESSOR_INPUT_VIEW_DESC_0, D3D11_VIDEO_PROCESSOR_OUTPUT_VIEW_DESC,
        D3D11_VIDEO_PROCESSOR_OUTPUT_VIEW_DESC_0, D3D11_VIDEO_PROCESSOR_STREAM,
        D3D11_VIDEO_USAGE_PLAYBACK_NORMAL, D3D11_VPIV_DIMENSION_TEXTURE2D,
        D3D11_VPOV_DIMENSION_TEXTURE2D, ID3D11Multithread, ID3D11Texture2D, ID3D11VideoContext,
        ID3D11VideoDevice, ID3D11VideoProcessor, ID3D11VideoProcessorEnumerator,
        ID3D11VideoProcessorOutputView,
    },
    Dxgi::Common::{DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_FORMAT_NV12, DXGI_RATIONAL, DXGI_SAMPLE_DESC},
};
use windows::core::Interface;

use crate::ComposeError;

/// Converts captured BGRA frames to encoder-ready NV12 on the capture device's GPU.
pub struct GpuFrameConverter {
    context: ID3D11VideoContext,
    enumerator: ID3D11VideoProcessorEnumerator,
    processor: ID3D11VideoProcessor,
    output: ID3D11Texture2D,
    output_view: ID3D11VideoProcessorOutputView,
    input_width: u32,
    input_height: u32,
    output_width: u32,
    output_height: u32,
}

impl GpuFrameConverter {
    pub fn new(
        source: &ID3D11Texture2D,
        output_width: u32,
        output_height: u32,
        frame_rate: u32,
    ) -> Result<Self, ComposeError> {
        let mut source_desc = D3D11_TEXTURE2D_DESC::default();
        unsafe { source.GetDesc(&mut source_desc) };
        if output_width == 0
            || output_height == 0
            || output_width > 1920
            || output_height > 1080
            || output_width % 2 != 0
            || output_height % 2 != 0
            || !(1..=60).contains(&frame_rate)
            || source_desc.Format != DXGI_FORMAT_B8G8R8A8_UNORM
        {
            return Err(ComposeError::NotInitialized);
        }

        let device = unsafe { source.GetDevice() }.map_err(backend)?;
        if source_desc.Width == 0 || source_desc.Height == 0 {
            return Err(ComposeError::NotInitialized);
        }
        let video_device: ID3D11VideoDevice = device.cast().map_err(backend)?;
        let context = unsafe { device.GetImmediateContext() }.map_err(backend)?;
        let multithread: ID3D11Multithread = context.cast().map_err(backend)?;
        let _ = unsafe { multithread.SetMultithreadProtected(true) };
        let video_context: ID3D11VideoContext = context.cast().map_err(backend)?;
        let content = D3D11_VIDEO_PROCESSOR_CONTENT_DESC {
            InputFrameFormat: D3D11_VIDEO_FRAME_FORMAT_PROGRESSIVE,
            InputFrameRate: DXGI_RATIONAL {
                Numerator: frame_rate,
                Denominator: 1,
            },
            InputWidth: source_desc.Width,
            InputHeight: source_desc.Height,
            OutputFrameRate: DXGI_RATIONAL {
                Numerator: frame_rate,
                Denominator: 1,
            },
            OutputWidth: output_width,
            OutputHeight: output_height,
            Usage: D3D11_VIDEO_USAGE_PLAYBACK_NORMAL,
        };
        let enumerator =
            unsafe { video_device.CreateVideoProcessorEnumerator(&content) }.map_err(backend)?;
        let input_support =
            unsafe { enumerator.CheckVideoProcessorFormat(DXGI_FORMAT_B8G8R8A8_UNORM) }
                .map_err(backend)?;
        let output_support =
            unsafe { enumerator.CheckVideoProcessorFormat(DXGI_FORMAT_NV12) }.map_err(backend)?;
        if input_support & D3D11_VIDEO_PROCESSOR_FORMAT_SUPPORT_INPUT.0 as u32 == 0
            || output_support & D3D11_VIDEO_PROCESSOR_FORMAT_SUPPORT_OUTPUT.0 as u32 == 0
        {
            return Err(ComposeError::Backend(
                "This GPU cannot convert captured BGRA frames to NV12".into(),
            ));
        }
        let processor =
            unsafe { video_device.CreateVideoProcessor(&enumerator, 0) }.map_err(backend)?;
        let mut output = None;
        unsafe {
            device.CreateTexture2D(
                &D3D11_TEXTURE2D_DESC {
                    Width: output_width,
                    Height: output_height,
                    MipLevels: 1,
                    ArraySize: 1,
                    Format: DXGI_FORMAT_NV12,
                    SampleDesc: DXGI_SAMPLE_DESC {
                        Count: 1,
                        Quality: 0,
                    },
                    Usage: D3D11_USAGE_DEFAULT,
                    BindFlags: D3D11_BIND_RENDER_TARGET.0 as u32,
                    CPUAccessFlags: 0,
                    MiscFlags: 0,
                },
                None,
                Some(&mut output),
            )
        }
        .map_err(backend)?;
        let output = output
            .ok_or_else(|| ComposeError::Backend("D3D11 returned no NV12 output texture".into()))?;
        let output_view_desc = D3D11_VIDEO_PROCESSOR_OUTPUT_VIEW_DESC {
            ViewDimension: D3D11_VPOV_DIMENSION_TEXTURE2D,
            Anonymous: D3D11_VIDEO_PROCESSOR_OUTPUT_VIEW_DESC_0 {
                Texture2D: D3D11_TEX2D_VPOV { MipSlice: 0 },
            },
        };
        let mut output_view = None;
        unsafe {
            video_device.CreateVideoProcessorOutputView(
                &output,
                &enumerator,
                &output_view_desc,
                Some(&mut output_view),
            )
        }
        .map_err(backend)?;
        let output_view = output_view
            .ok_or_else(|| ComposeError::Backend("D3D11 returned no NV12 output view".into()))?;

        Ok(Self {
            context: video_context,
            enumerator,
            processor,
            output,
            output_view,
            input_width: source_desc.Width,
            input_height: source_desc.Height,
            output_width,
            output_height,
        })
    }

    pub fn convert(&mut self, source: &ID3D11Texture2D) -> Result<ID3D11Texture2D, ComposeError> {
        let mut desc = D3D11_TEXTURE2D_DESC::default();
        unsafe { source.GetDesc(&mut desc) };
        if desc.Width != self.input_width
            || desc.Height != self.input_height
            || desc.Format != DXGI_FORMAT_B8G8R8A8_UNORM
        {
            return Err(ComposeError::Backend(
                "Captured frame dimensions or format changed; rebuild the GPU converter".into(),
            ));
        }

        let input_desc = D3D11_VIDEO_PROCESSOR_INPUT_VIEW_DESC {
            FourCC: 0,
            ViewDimension: D3D11_VPIV_DIMENSION_TEXTURE2D,
            Anonymous: D3D11_VIDEO_PROCESSOR_INPUT_VIEW_DESC_0 {
                Texture2D: D3D11_TEX2D_VPIV {
                    MipSlice: 0,
                    ArraySlice: 0,
                },
            },
        };
        let device: ID3D11VideoDevice = unsafe { source.GetDevice() }
            .map_err(backend)?
            .cast()
            .map_err(backend)?;
        let mut input = None;
        unsafe {
            device.CreateVideoProcessorInputView(
                source,
                &self.enumerator,
                &input_desc,
                Some(&mut input),
            )
        }
        .map_err(backend)?;
        let input = input.ok_or_else(|| {
            ComposeError::Backend("D3D11 returned no captured-frame input view".into())
        })?;
        let mut destination = fit_rect(
            self.input_width,
            self.input_height,
            self.output_width,
            self.output_height,
        );
        unsafe {
            self.context.VideoProcessorSetStreamDestRect(
                &self.processor,
                0,
                true,
                Some(&mut destination),
            )
        };
        let mut stream = D3D11_VIDEO_PROCESSOR_STREAM {
            Enable: true.into(),
            pInputSurface: ManuallyDrop::new(Some(input)),
            ..Default::default()
        };
        let result = unsafe {
            self.context
                .VideoProcessorBlt(&self.processor, &self.output_view, 0, &[stream.clone()])
        };
        unsafe { ManuallyDrop::drop(&mut stream.pInputSurface) };
        result.map_err(backend)?;
        Ok(self.output.clone())
    }

    pub fn output_size(&self) -> (u32, u32) {
        (self.output_width, self.output_height)
    }
}

fn fit_rect(
    source_width: u32,
    source_height: u32,
    output_width: u32,
    output_height: u32,
) -> windows::Win32::Foundation::RECT {
    let scale = (output_width as f64 / source_width as f64)
        .min(output_height as f64 / source_height as f64);
    let width = (source_width as f64 * scale).round() as i32;
    let height = (source_height as f64 * scale).round() as i32;
    let x = (output_width as i32 - width) / 2;
    let y = (output_height as i32 - height) / 2;
    windows::Win32::Foundation::RECT {
        left: x,
        top: y,
        right: x + width,
        bottom: y + height,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_source_aspect_inside_output_canvas() {
        assert_eq!(
            fit_rect(1920, 1080, 1280, 720),
            windows::Win32::Foundation::RECT {
                left: 0,
                top: 0,
                right: 1280,
                bottom: 720,
            }
        );
        assert_eq!(
            fit_rect(1440, 1080, 1920, 1080),
            windows::Win32::Foundation::RECT {
                left: 240,
                top: 0,
                right: 1680,
                bottom: 1080,
            }
        );
    }
}

fn backend(error: windows::core::Error) -> ComposeError {
    ComposeError::Backend(error.to_string())
}
