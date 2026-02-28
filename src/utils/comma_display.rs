use num_format::{Locale, ToFormattedString};

// カンマ変換用拡張関数
pub trait CommaDisplay {
    /// カンマ区切りの文字列を返す。
    /// 浮動小数点数の場合、デフォルトの精度が適用されるか、
    /// 整数部のみの整形を出力。
    fn to_comma(&self) -> String;

    /// 表示小数点以下桁を指定してカンマ区切り文字列を返す。
    /// 整数の場合は精度指定を無視してそのまま返す。
    fn to_comma_fmt(&self, precision: usize) -> String;
}

// 整数型用マクロ
macro_rules! impl_comma_for_int {
    ($($t:ty),*) => {
        $(
            impl CommaDisplay for $t {
                fn to_comma(&self) -> String {
                    self.to_formatted_string(&Locale::en)
                }

                fn to_comma_fmt(&self, _precision: usize) -> String {
                    // 整数なので精度指定は無視してそのまま返す
                    self.to_comma()
                }
            }
        )*
    };
}
impl_comma_for_int!(i8, i16, i32, i64, i128, isize);
impl_comma_for_int!(u8, u16, u32, u64, u128, usize);

//  浮動小数点型の設定
impl CommaDisplay for f64 {
    fn to_comma(&self) -> String {
        self.to_comma_fmt(2)
    }

    fn to_comma_fmt(&self, precision: usize) -> String {
        let s = format!("{:.1$}", self, precision);
        format_float_string(&s)
    }
}

impl CommaDisplay for f32 {
    fn to_comma(&self) -> String {
        self.to_comma_fmt(2)
    }

    fn to_comma_fmt(&self, precision: usize) -> String {
        let s = format!("{:.1$}", self, precision);
        format_float_string(&s)
    }
}

// ヘルパー
fn format_float_string(s: &str) -> String {
    let parts: Vec<&str> = s.split('.').collect();
    let int_part_str = parts[0];
    let dec_part = if parts.len() > 1 { parts[1] } else { "" };

    // 整数部をパースして整形
    match int_part_str.parse::<i64>() {
        Ok(int_val) => {
            let formatted_int = int_val.to_formatted_string(&Locale::en);
            if !dec_part.is_empty() {
                format!("{}.{}", formatted_int, dec_part)
            } else {
                formatted_int
            }
        }
        Err(_) => s.to_string(), // NaNなどはそのまま返す
    }
}

#[cfg(test)]
mod tests {
    use super::CommaDisplay;

    #[test]
    fn integer_formats_with_commas() {
        // 整数値が 3 桁区切りのカンマ形式で表示されることを確認する。
        assert_eq!(1234567_i64.to_comma(), "1,234,567");
        assert_eq!((-987654_i64).to_comma(), "-987,654");
    }

    #[test]
    fn float_formats_with_specified_precision() {
        // 浮動小数点数が指定精度で丸められカンマ付き表示になることを確認する。
        assert_eq!(12345.6789_f64.to_comma_fmt(2), "12,345.68");
        assert_eq!((-1234.5_f64).to_comma_fmt(3), "-1,234.500");
    }

    #[test]
    fn float_default_precision_is_two_digits() {
        // to_comma の既定精度が小数点以下 2 桁であることを確認する。
        assert_eq!(12.3_f64.to_comma(), "12.30");
        assert_eq!(12.3_f32.to_comma(), "12.30");
    }

    #[test]
    fn nan_is_returned_as_is() {
        // NaN は数値変換せず文字列 "NaN" として返されることを確認する。
        assert_eq!(f64::NAN.to_comma_fmt(2), "NaN");
    }
}
