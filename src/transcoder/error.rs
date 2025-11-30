//! FFmpegエラー解析

/// FFmpegエラーの種類
#[derive(Debug, Clone, PartialEq)]
pub enum FfmpegErrorKind {
    /// エンコーダーがサポートされていない
    EncoderNotSupported(String),
    /// デコーダーがサポートされていない
    DecoderNotSupported(String),
    /// HWアクセラレーションが利用できない
    HwAccelNotAvailable(String),
    /// 入力ファイルが見つからない
    InputNotFound,
    /// 入力ファイルが破損している
    InputCorrupted,
    /// 出力先に書き込めない
    OutputWriteError,
    /// ディスク容量不足
    DiskFull,
    /// メモリ不足
    OutOfMemory,
    /// 権限エラー
    PermissionDenied,
    /// コーデックオプションが無効
    InvalidCodecOption(String),
    /// 不明なエラー
    Unknown(String),
}

/// FFmpegエラー解析結果
#[derive(Debug, Clone)]
pub struct FfmpegError {
    /// エラーの種類
    pub kind: FfmpegErrorKind,
    /// ユーザー向けメッセージ
    pub user_message: String,
    /// 解決策の提案
    pub suggestion: Option<String>,
    /// 元のエラーメッセージ
    pub raw_message: String,
}

impl FfmpegError {
    /// FFmpegのstderrからエラーを解析
    pub fn parse(stderr: &str) -> Self {
        let stderr_lower = stderr.to_lowercase();

        // エンコーダーがサポートされていない
        if stderr_lower.contains("unknown encoder")
            || stderr_lower.contains("encoder") && stderr_lower.contains("not found")
            || stderr_lower.contains("no such encoder")
        {
            let encoder = Self::extract_encoder_name(stderr);
            return Self::encoder_not_supported(&encoder, stderr);
        }

        // 特定のHWエンコーダーエラー（より具体的なエラーメッセージを先にチェック）
        // Intel QSV関連エラー
        if stderr_lower.contains("no qsv-supporting device")
            || stderr_lower.contains("device creation failed") && stderr_lower.contains("qsv")
            || stderr_lower.contains("mfx")
                && (stderr_lower.contains("error") || stderr_lower.contains("failed"))
            || stderr_lower.contains("h264_qsv")
                && (stderr_lower.contains("error")
                    || stderr_lower.contains("failed")
                    || stderr_lower.contains("not found"))
            || stderr_lower.contains("hevc_qsv")
                && (stderr_lower.contains("error")
                    || stderr_lower.contains("failed")
                    || stderr_lower.contains("not found"))
            || stderr_lower.contains("av1_qsv")
                && (stderr_lower.contains("error")
                    || stderr_lower.contains("failed")
                    || stderr_lower.contains("not found"))
            || stderr_lower.contains("libmfx") && stderr_lower.contains("not found")
            || stderr_lower.contains("qsv") && stderr_lower.contains("init")
        {
            return Self::hwaccel_not_available("Intel QSV", stderr);
        }

        // NVIDIA NVENC関連エラー
        if stderr_lower.contains("no nvenc capable devices found")
            || stderr_lower.contains("cannot load nvcuda.dll")
            || stderr_lower.contains("cannot load nvencodeapi")
            || stderr_lower.contains("h264_nvenc")
                && (stderr_lower.contains("error")
                    || stderr_lower.contains("failed")
                    || stderr_lower.contains("not found"))
            || stderr_lower.contains("hevc_nvenc")
                && (stderr_lower.contains("error")
                    || stderr_lower.contains("failed")
                    || stderr_lower.contains("not found"))
        {
            return Self::hwaccel_not_available("NVIDIA NVENC", stderr);
        }

        // AMD AMF関連エラー
        if stderr_lower.contains("amf failed")
            || stderr_lower.contains("no amf capable device")
            || stderr_lower.contains("h264_amf")
                && (stderr_lower.contains("error")
                    || stderr_lower.contains("failed")
                    || stderr_lower.contains("not found"))
            || stderr_lower.contains("hevc_amf")
                && (stderr_lower.contains("error")
                    || stderr_lower.contains("failed")
                    || stderr_lower.contains("not found"))
        {
            return Self::hwaccel_not_available("AMD AMF", stderr);
        }

        // HWアクセラレーションエラー（一般的なパターン - 上記で判定されなかった場合）
        if stderr_lower.contains("nvenc")
            || stderr_lower.contains("qsv")
            || stderr_lower.contains("amf")
            || stderr_lower.contains("cuda")
            || stderr_lower.contains("d3d11")
            || stderr_lower.contains("vaapi")
        {
            if stderr_lower.contains("cannot load")
                || stderr_lower.contains("failed to")
                || stderr_lower.contains("not found")
                || stderr_lower.contains("unavailable")
                || stderr_lower.contains("no capable devices")
            {
                let hwaccel = Self::extract_hwaccel_name(stderr);
                return Self::hwaccel_not_available(&hwaccel, stderr);
            }
        }

        // デコーダーがサポートされていない
        if stderr_lower.contains("decoder") && stderr_lower.contains("not found")
            || stderr_lower.contains("unknown decoder")
        {
            let decoder = Self::extract_decoder_name(stderr);
            return Self::decoder_not_supported(&decoder, stderr);
        }

        // 入力ファイル関連
        if stderr_lower.contains("no such file")
            || stderr_lower.contains("does not exist")
            || stderr_lower.contains("file not found")
        {
            return Self::input_not_found(stderr);
        }

        if stderr_lower.contains("invalid data found")
            || stderr_lower.contains("corrupt")
            || stderr_lower.contains("moov atom not found")
            || stderr_lower.contains("end of file") && stderr_lower.contains("invalid")
        {
            return Self::input_corrupted(stderr);
        }

        // 出力関連
        if stderr_lower.contains("permission denied") || stderr_lower.contains("access denied") {
            return Self::permission_denied(stderr);
        }

        if stderr_lower.contains("no space left")
            || stderr_lower.contains("disk full")
            || stderr_lower.contains("not enough space")
        {
            return Self::disk_full(stderr);
        }

        if stderr_lower.contains("cannot open")
            && (stderr_lower.contains("output") || stderr_lower.contains("writing"))
        {
            return Self::output_write_error(stderr);
        }

        // メモリ関連
        if stderr_lower.contains("out of memory")
            || stderr_lower.contains("memory allocation failed")
            || stderr_lower.contains("cannot allocate")
        {
            return Self::out_of_memory(stderr);
        }

        // コーデックオプション関連
        if stderr_lower.contains("option") && stderr_lower.contains("not found")
            || stderr_lower.contains("unrecognized option")
            || stderr_lower.contains("invalid option")
        {
            let option = Self::extract_option_name(stderr);
            return Self::invalid_codec_option(&option, stderr);
        }

        // 不明なエラー
        Self::unknown(stderr)
    }

    /// エンコーダーがサポートされていないエラーを作成
    fn encoder_not_supported(encoder: &str, raw: &str) -> Self {
        let display_name = Self::get_encoder_display_name(encoder);
        Self {
            kind: FfmpegErrorKind::EncoderNotSupported(encoder.to_string()),
            user_message: format!(
                "エンコーダー「{}」はこのシステムでサポートされていません",
                display_name
            ),
            suggestion: Some(Self::get_encoder_suggestion(encoder)),
            raw_message: raw.to_string(),
        }
    }

    /// HWアクセラレーションが利用できないエラーを作成
    fn hwaccel_not_available(hwaccel: &str, raw: &str) -> Self {
        Self {
            kind: FfmpegErrorKind::HwAccelNotAvailable(hwaccel.to_string()),
            user_message: format!(
                "ハードウェアアクセラレーション「{}」が利用できません",
                hwaccel
            ),
            suggestion: Some(
                "HWアクセラレーション設定を「ソフトウェア」に変更するか、\
                 グラフィックドライバーを最新版に更新してください"
                    .to_string(),
            ),
            raw_message: raw.to_string(),
        }
    }

    /// デコーダーがサポートされていないエラーを作成
    fn decoder_not_supported(decoder: &str, raw: &str) -> Self {
        Self {
            kind: FfmpegErrorKind::DecoderNotSupported(decoder.to_string()),
            user_message: format!(
                "入力ファイルのコーデック「{}」はサポートされていません",
                decoder
            ),
            suggestion: Some(
                "この入力形式をサポートするFFmpegビルドが必要です。\
                 GPLビルドのFFmpegをお試しください"
                    .to_string(),
            ),
            raw_message: raw.to_string(),
        }
    }

    /// 入力ファイルが見つからないエラーを作成
    fn input_not_found(raw: &str) -> Self {
        Self {
            kind: FfmpegErrorKind::InputNotFound,
            user_message: "入力ファイルが見つかりません".to_string(),
            suggestion: Some("ファイルが移動または削除されていないか確認してください".to_string()),
            raw_message: raw.to_string(),
        }
    }

    /// 入力ファイルが破損しているエラーを作成
    fn input_corrupted(raw: &str) -> Self {
        Self {
            kind: FfmpegErrorKind::InputCorrupted,
            user_message: "入力ファイルが破損しているか、形式が不正です".to_string(),
            suggestion: Some(
                "ファイルが正常に再生できるか確認してください。\
                 ダウンロードが途中で中断された可能性があります"
                    .to_string(),
            ),
            raw_message: raw.to_string(),
        }
    }

    /// 権限エラーを作成
    fn permission_denied(raw: &str) -> Self {
        Self {
            kind: FfmpegErrorKind::PermissionDenied,
            user_message: "ファイルへのアクセス権限がありません".to_string(),
            suggestion: Some(
                "出力先フォルダへの書き込み権限があるか確認してください。\
                 管理者権限が必要な場合があります"
                    .to_string(),
            ),
            raw_message: raw.to_string(),
        }
    }

    /// ディスク容量不足エラーを作成
    fn disk_full(raw: &str) -> Self {
        Self {
            kind: FfmpegErrorKind::DiskFull,
            user_message: "ディスク容量が不足しています".to_string(),
            suggestion: Some("出力先ドライブの空き容量を確保してください".to_string()),
            raw_message: raw.to_string(),
        }
    }

    /// 出力書き込みエラーを作成
    fn output_write_error(raw: &str) -> Self {
        Self {
            kind: FfmpegErrorKind::OutputWriteError,
            user_message: "出力ファイルを作成できません".to_string(),
            suggestion: Some(
                "出力先フォルダが存在し、書き込み可能であることを確認してください".to_string(),
            ),
            raw_message: raw.to_string(),
        }
    }

    /// メモリ不足エラーを作成
    fn out_of_memory(raw: &str) -> Self {
        Self {
            kind: FfmpegErrorKind::OutOfMemory,
            user_message: "メモリが不足しています".to_string(),
            suggestion: Some(
                "他のアプリケーションを終了するか、解像度を下げてお試しください".to_string(),
            ),
            raw_message: raw.to_string(),
        }
    }

    /// 無効なコーデックオプションエラーを作成
    fn invalid_codec_option(option: &str, raw: &str) -> Self {
        Self {
            kind: FfmpegErrorKind::InvalidCodecOption(option.to_string()),
            user_message: format!("コーデックオプション「{}」が無効です", option),
            suggestion: Some(
                "選択したエンコーダーはこのオプションをサポートしていません。\
                 設定を変更してお試しください"
                    .to_string(),
            ),
            raw_message: raw.to_string(),
        }
    }

    /// 不明なエラーを作成
    fn unknown(raw: &str) -> Self {
        // 最後の有意なエラー行を抽出
        let error_line = raw
            .lines()
            .filter(|line| {
                let lower = line.to_lowercase();
                lower.contains("error")
                    || lower.contains("failed")
                    || lower.contains("cannot")
                    || lower.contains("unable")
            })
            .last()
            .unwrap_or("変換中にエラーが発生しました");

        Self {
            kind: FfmpegErrorKind::Unknown(error_line.to_string()),
            user_message: format!("変換エラー: {}", Self::truncate_message(error_line, 100)),
            suggestion: None,
            raw_message: raw.to_string(),
        }
    }

    /// エンコーダー名を抽出
    fn extract_encoder_name(stderr: &str) -> String {
        // "Unknown encoder 'xxx'" や "Encoder xxx not found" などからエンコーダー名を抽出
        for line in stderr.lines() {
            let lower = line.to_lowercase();
            if lower.contains("encoder") {
                // シングルクォートで囲まれた名前を探す
                if let Some(start) = line.find('\'') {
                    if let Some(end) = line[start + 1..].find('\'') {
                        return line[start + 1..start + 1 + end].to_string();
                    }
                }
                // スペースで区切られた名前を探す
                let words: Vec<&str> = line.split_whitespace().collect();
                for (i, word) in words.iter().enumerate() {
                    if word.to_lowercase() == "encoder" && i + 1 < words.len() {
                        let name = words[i + 1].trim_matches(|c| c == '\'' || c == '"');
                        if !name.is_empty() && name != "not" {
                            return name.to_string();
                        }
                    }
                }
            }
        }
        "不明".to_string()
    }

    /// デコーダー名を抽出
    fn extract_decoder_name(stderr: &str) -> String {
        for line in stderr.lines() {
            let lower = line.to_lowercase();
            if lower.contains("decoder") {
                if let Some(start) = line.find('\'') {
                    if let Some(end) = line[start + 1..].find('\'') {
                        return line[start + 1..start + 1 + end].to_string();
                    }
                }
            }
        }
        "不明".to_string()
    }

    /// HWアクセラレーション名を抽出
    /// 注意: エラーメッセージに複数のHWアクセラレーション名が含まれる可能性があるため、
    /// より具体的なエンコーダー名（h264_qsv等）を優先してチェックする
    fn extract_hwaccel_name(stderr: &str) -> String {
        let lower = stderr.to_lowercase();

        // 具体的なエンコーダー名を優先的にチェック（より正確な判定のため）
        // QSVエンコーダー名をチェック
        if lower.contains("h264_qsv")
            || lower.contains("hevc_qsv")
            || lower.contains("av1_qsv")
            || lower.contains("vp9_qsv")
        {
            return "Intel QSV".to_string();
        }
        // AMFエンコーダー名をチェック
        if lower.contains("h264_amf") || lower.contains("hevc_amf") || lower.contains("av1_amf") {
            return "AMD AMF".to_string();
        }
        // NVENCエンコーダー名をチェック
        if lower.contains("h264_nvenc")
            || lower.contains("hevc_nvenc")
            || lower.contains("av1_nvenc")
        {
            return "NVIDIA NVENC".to_string();
        }

        // 一般的なキーワードでチェック（エンコーダー名が見つからない場合のフォールバック）
        // QSV特有のエラーキーワード
        if lower.contains("qsv") || lower.contains("quick sync") || lower.contains("mfx") {
            "Intel QSV".to_string()
        } else if lower.contains("amf") || lower.contains("advanced media framework") {
            "AMD AMF".to_string()
        } else if lower.contains("nvenc") || lower.contains("cuda") || lower.contains("nvcuda") {
            "NVIDIA NVENC".to_string()
        } else if lower.contains("vaapi") {
            "VAAPI".to_string()
        } else {
            "ハードウェアアクセラレーション".to_string()
        }
    }

    /// オプション名を抽出
    fn extract_option_name(stderr: &str) -> String {
        for line in stderr.lines() {
            if line.to_lowercase().contains("option") {
                if let Some(start) = line.find('\'') {
                    if let Some(end) = line[start + 1..].find('\'') {
                        return line[start + 1..start + 1 + end].to_string();
                    }
                }
            }
        }
        "不明".to_string()
    }

    /// エンコーダーの表示名を取得
    fn get_encoder_display_name(encoder: &str) -> String {
        match encoder {
            "h264_nvenc" => "H.264 NVENC (NVIDIA)".to_string(),
            "hevc_nvenc" => "H.265/HEVC NVENC (NVIDIA)".to_string(),
            "av1_nvenc" => "AV1 NVENC (NVIDIA)".to_string(),
            "h264_qsv" => "H.264 QSV (Intel)".to_string(),
            "hevc_qsv" => "H.265/HEVC QSV (Intel)".to_string(),
            "av1_qsv" => "AV1 QSV (Intel)".to_string(),
            "vp9_qsv" => "VP9 QSV (Intel)".to_string(),
            "h264_amf" => "H.264 AMF (AMD)".to_string(),
            "hevc_amf" => "H.265/HEVC AMF (AMD)".to_string(),
            "av1_amf" => "AV1 AMF (AMD)".to_string(),
            "libx264" => "H.264 (ソフトウェア)".to_string(),
            "libx265" => "H.265/HEVC (ソフトウェア)".to_string(),
            "libvpx-vp9" => "VP9 (ソフトウェア)".to_string(),
            "libsvtav1" => "AV1 (ソフトウェア)".to_string(),
            "aac" => "AAC オーディオ".to_string(),
            "libmp3lame" => "MP3 オーディオ".to_string(),
            "flac" => "FLAC オーディオ".to_string(),
            _ => encoder.to_string(),
        }
    }

    /// エンコーダーに応じた解決策を取得
    fn get_encoder_suggestion(encoder: &str) -> String {
        if encoder.contains("nvenc") {
            "NVIDIAグラフィックカードが必要です。\
             HWアクセラレーション設定を「ソフトウェア」に変更してください"
                .to_string()
        } else if encoder.contains("qsv") {
            "Intel製CPUの内蔵グラフィックスが必要です。\
             HWアクセラレーション設定を「ソフトウェア」に変更してください"
                .to_string()
        } else if encoder.contains("amf") {
            "AMDグラフィックカードが必要です。\
             HWアクセラレーション設定を「ソフトウェア」に変更してください"
                .to_string()
        } else if encoder.contains("libsvtav1") || encoder.contains("av1") {
            "AV1エンコーダーがインストールされていません。\
             H.264またはH.265コーデックをお試しください"
                .to_string()
        } else if encoder.contains("libmp3lame") {
            "MP3エンコーダー(LAME)がインストールされていません。\
             AACオーディオコーデックをお試しください"
                .to_string()
        } else {
            "別のコーデックまたはHWアクセラレーション設定をお試しください".to_string()
        }
    }

    /// メッセージを切り詰める
    fn truncate_message(msg: &str, max_len: usize) -> String {
        if msg.len() <= max_len {
            msg.to_string()
        } else {
            format!("{}...", &msg[..max_len])
        }
    }

    /// ユーザー向けの完全なエラーメッセージを生成
    pub fn format_user_message(&self) -> String {
        let mut msg = self.user_message.clone();
        if let Some(ref suggestion) = self.suggestion {
            msg.push_str("\n\n💡 ");
            msg.push_str(suggestion);
        }
        msg
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_encoder_not_found() {
        let stderr = "Unknown encoder 'h264_nvenc'";
        let error = FfmpegError::parse(stderr);
        assert!(matches!(
            error.kind,
            FfmpegErrorKind::EncoderNotSupported(_)
        ));
    }

    #[test]
    fn test_parse_nvenc_not_available() {
        let stderr = "Cannot load nvcuda.dll";
        let error = FfmpegError::parse(stderr);
        assert!(matches!(
            error.kind,
            FfmpegErrorKind::HwAccelNotAvailable(_)
        ));
    }

    #[test]
    fn test_parse_input_not_found() {
        let stderr = "No such file or directory";
        let error = FfmpegError::parse(stderr);
        assert!(matches!(error.kind, FfmpegErrorKind::InputNotFound));
    }

    #[test]
    fn test_parse_permission_denied() {
        let stderr = "Permission denied";
        let error = FfmpegError::parse(stderr);
        assert!(matches!(error.kind, FfmpegErrorKind::PermissionDenied));
    }
}
