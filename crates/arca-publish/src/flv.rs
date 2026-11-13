use bytes::Bytes;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum FlvMuxError {
    #[error("Codec parameter sets must be non-empty and fit the FLV AVC length fields")]
    InvalidParameterSet,
    #[error("AVC frames need at least one NAL unit")]
    EmptyVideoFrame,
    #[error("AVC NAL units must be raw and must not include Annex B start codes")]
    AnnexBNotSupported,
    #[error("H.264 Annex B data contains no valid NAL units")]
    InvalidAnnexB,
    #[error("AAC codec configuration must contain at least two bytes")]
    InvalidAacConfiguration,
    #[error("AAC frame is empty")]
    EmptyAudioFrame,
}

/// Creates an FLV AVC sequence-header body from the encoder's SPS and PPS NAL units.
pub fn avc_sequence_header(sps: &[u8], pps: &[u8]) -> Result<Bytes, FlvMuxError> {
    if sps.len() < 4
        || pps.is_empty()
        || sps.len() > u16::MAX as usize
        || pps.len() > u16::MAX as usize
    {
        return Err(FlvMuxError::InvalidParameterSet);
    }
    reject_annex_b(sps)?;
    reject_annex_b(pps)?;

    let mut body = Vec::with_capacity(16 + sps.len() + pps.len());
    body.extend_from_slice(&[0x17, 0x00, 0, 0, 0]);
    body.extend_from_slice(&[1, sps[1], sps[2], sps[3], 0xFF, 0xE1]);
    body.extend_from_slice(&(sps.len() as u16).to_be_bytes());
    body.extend_from_slice(sps);
    body.push(1);
    body.extend_from_slice(&(pps.len() as u16).to_be_bytes());
    body.extend_from_slice(pps);
    Ok(Bytes::from(body))
}

/// Creates an FLV AVC NALU packet body. NAL units are raw payloads without Annex B start codes.
pub fn avc_frame(nalus: &[&[u8]], keyframe: bool) -> Result<Bytes, FlvMuxError> {
    if nalus.is_empty() {
        return Err(FlvMuxError::EmptyVideoFrame);
    }
    let mut body = Vec::new();
    body.extend_from_slice(&[if keyframe { 0x17 } else { 0x27 }, 0x01, 0, 0, 0]);
    for nalu in nalus {
        if nalu.is_empty() {
            return Err(FlvMuxError::EmptyVideoFrame);
        }
        reject_annex_b(nalu)?;
        let length = u32::try_from(nalu.len()).map_err(|_| FlvMuxError::InvalidParameterSet)?;
        body.extend_from_slice(&length.to_be_bytes());
        body.extend_from_slice(nalu);
    }
    Ok(Bytes::from(body))
}

/// Converts an Annex B access unit (as emitted by Media Foundation) to an FLV AVC frame.
pub fn avc_frame_from_annex_b(access_unit: &[u8], keyframe: bool) -> Result<Bytes, FlvMuxError> {
    avc_frame(&annex_b_nalus(access_unit)?, keyframe)
}

/// Builds an FLV AVC sequence header from Media Foundation's Annex B SPS/PPS blob.
pub fn avc_sequence_header_from_annex_b(blob: &[u8]) -> Result<Bytes, FlvMuxError> {
    let mut sps = None;
    let mut pps = None;
    for nalu in annex_b_nalus(blob)? {
        match nalu[0] & 0x1f {
            7 => sps = Some(nalu),
            8 => pps = Some(nalu),
            _ => {}
        }
    }
    avc_sequence_header(
        sps.ok_or(FlvMuxError::InvalidAnnexB)?,
        pps.ok_or(FlvMuxError::InvalidAnnexB)?,
    )
}

fn annex_b_nalus(data: &[u8]) -> Result<Vec<&[u8]>, FlvMuxError> {
    fn start_code(data: &[u8], from: usize) -> Option<(usize, usize)> {
        (from..data.len().saturating_sub(2)).find_map(|i| {
            if data[i..].starts_with(&[0, 0, 0, 1]) {
                Some((i, 4))
            } else if data[i..].starts_with(&[0, 0, 1]) {
                Some((i, 3))
            } else {
                None
            }
        })
    }

    let mut nalus = Vec::new();
    let mut current = start_code(data, 0).ok_or(FlvMuxError::InvalidAnnexB)?;
    loop {
        let begin = current.0 + current.1;
        let next = start_code(data, begin);
        let end = next.map_or(data.len(), |(index, _)| index);
        let nalu = data[begin..end]
            .strip_suffix(&[0])
            .unwrap_or(&data[begin..end]);
        if nalu.is_empty() {
            return Err(FlvMuxError::InvalidAnnexB);
        }
        nalus.push(nalu);
        let Some(next) = next else { break };
        current = next;
    }
    Ok(nalus)
}

/// Creates an FLV AAC sequence-header body from the encoder's AudioSpecificConfig bytes.
pub fn aac_sequence_header(config: &[u8]) -> Result<Bytes, FlvMuxError> {
    if config.len() < 2 {
        return Err(FlvMuxError::InvalidAacConfiguration);
    }
    let mut body = Vec::with_capacity(2 + config.len());
    body.extend_from_slice(&[0xAF, 0x00]);
    body.extend_from_slice(config);
    Ok(Bytes::from(body))
}

/// Creates an FLV AAC raw-frame body.
pub fn aac_frame(frame: &[u8]) -> Result<Bytes, FlvMuxError> {
    if frame.is_empty() {
        return Err(FlvMuxError::EmptyAudioFrame);
    }
    let mut body = Vec::with_capacity(2 + frame.len());
    body.extend_from_slice(&[0xAF, 0x01]);
    body.extend_from_slice(frame);
    Ok(Bytes::from(body))
}

fn reject_annex_b(nalu: &[u8]) -> Result<(), FlvMuxError> {
    if nalu.starts_with(&[0, 0, 1]) || nalu.starts_with(&[0, 0, 0, 1]) {
        Err(FlvMuxError::AnnexBNotSupported)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_avc_sequence_header_and_length_prefixed_keyframe() {
        let sps = [0x67, 0x64, 0x00, 0x28, 0xAA];
        let pps = [0x68, 0xEE, 0x3C, 0x80];
        let sequence = avc_sequence_header(&sps, &pps).unwrap();
        assert_eq!(&sequence[..6], &[0x17, 0x00, 0, 0, 0, 1]);

        let frame = avc_frame(&[&[0x65, 0x11, 0x22]], true).unwrap();
        assert_eq!(&frame[..9], &[0x17, 0x01, 0, 0, 0, 0, 0, 0, 3]);
        assert_eq!(&frame[9..], &[0x65, 0x11, 0x22]);
    }

    #[test]
    fn builds_aac_headers_and_rejects_annex_b_video() {
        assert_eq!(
            &aac_sequence_header(&[0x12, 0x10]).unwrap()[..],
            &[0xAF, 0, 0x12, 0x10]
        );
        assert_eq!(
            &aac_frame(&[0x01, 0x02]).unwrap()[..],
            &[0xAF, 1, 0x01, 0x02]
        );
        assert_eq!(
            avc_frame(&[&[0, 0, 0, 1, 0x65]], true),
            Err(FlvMuxError::AnnexBNotSupported)
        );
    }

    #[test]
    fn converts_media_foundation_annex_b_header_and_access_unit() {
        let header = [0, 0, 0, 1, 0x67, 0x64, 0, 0x28, 0xaa, 0, 0, 1, 0x68, 0xee];
        let sequence = avc_sequence_header_from_annex_b(&header).unwrap();
        assert_eq!(sequence[0..2], [0x17, 0]);

        let frame = avc_frame_from_annex_b(&[0, 0, 1, 0x65, 0x11, 0x22], true).unwrap();
        assert_eq!(&frame[..9], &[0x17, 1, 0, 0, 0, 0, 0, 0, 3]);
        assert_eq!(&frame[9..], &[0x65, 0x11, 0x22]);
    }

    #[test]
    fn rejects_annex_b_without_sps_and_pps() {
        assert_eq!(
            avc_sequence_header_from_annex_b(&[0, 0, 1, 0x65, 0x01]),
            Err(FlvMuxError::InvalidAnnexB)
        );
    }
}
