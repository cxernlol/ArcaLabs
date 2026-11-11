use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GpuInfo {
    pub name: String,
    pub vendor: String,
    pub suggested_encoder: String,
}

fn classify(name: &str) -> GpuInfo {
    let lower = name.to_ascii_lowercase();
    if lower.contains("nvidia") || lower.contains("geforce") || lower.contains("quadro") {
        return GpuInfo {
            name: name.to_string(),
            vendor: "nvidia".into(),
            suggested_encoder: "NVENC".into(),
        };
    }
    if lower.contains("amd") || lower.contains("radeon") || lower.contains("rx ") {
        return GpuInfo {
            name: name.to_string(),
            vendor: "amd".into(),
            suggested_encoder: "AMF".into(),
        };
    }
    if lower.contains("intel") || lower.contains("arc") || lower.contains("uhd") {
        return GpuInfo {
            name: name.to_string(),
            vendor: "intel".into(),
            suggested_encoder: "QuickSync".into(),
        };
    }
    GpuInfo {
        name: name.to_string(),
        vendor: "unknown".into(),
        suggested_encoder: "x264".into(),
    }
}

fn is_software_adapter(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower.contains("microsoft basic render")
        || lower.contains("remote desktop")
        || lower.contains("virtual")
}

#[cfg(windows)]
fn enumerate_adapter_names() -> Result<Vec<String>, String> {
    use windows::Win32::Graphics::Dxgi::{CreateDXGIFactory1, IDXGIFactory1};

    unsafe {
        let factory: IDXGIFactory1 =
            CreateDXGIFactory1().map_err(|e| format!("DXGI factory failed: {e}"))?;
        let mut names = Vec::new();
        let mut index = 0u32;
        loop {
            match factory.EnumAdapters1(index) {
                Ok(adapter) => {
                    let desc = adapter
                        .GetDesc1()
                        .map_err(|e| format!("DXGI adapter desc failed: {e}"))?;
                    let end = desc
                        .Description
                        .iter()
                        .position(|&c| c == 0)
                        .unwrap_or(desc.Description.len());
                    let name = String::from_utf16_lossy(&desc.Description[..end]);
                    names.push(name);
                    index += 1;
                }
                Err(_) => break,
            }
        }
        Ok(names)
    }
}

#[cfg(not(windows))]
fn enumerate_adapter_names() -> Result<Vec<String>, String> {
    Err("GPU detection is only available on Windows".into())
}

pub fn detect_gpu() -> Result<GpuInfo, String> {
    let names = enumerate_adapter_names()?;
    let preferred = names
        .into_iter()
        .filter(|n| !is_software_adapter(n))
        .collect::<Vec<_>>();

    if preferred.is_empty() {
        return Ok(GpuInfo {
            name: "No discrete GPU detected".into(),
            vendor: "unknown".into(),
            suggested_encoder: "x264".into(),
        });
    }

    for name in &preferred {
        let info = classify(name);
        if info.vendor == "nvidia" {
            return Ok(info);
        }
    }
    for name in &preferred {
        let info = classify(name);
        if info.vendor == "amd" {
            return Ok(info);
        }
    }
    Ok(classify(&preferred[0]))
}
