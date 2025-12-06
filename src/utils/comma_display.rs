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
