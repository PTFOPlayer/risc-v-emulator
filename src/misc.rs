#[macro_export]
macro_rules! fast_transmute {
    (<$start:expr, u16>, $data: expr ) => {
        unsafe { std::mem::transmute::<[u8; 2], u16>([$data[$start + 0], $data[$start + 1]]) }
    };
    (<$start:expr, u32>, $data: expr ) => {
        unsafe {
            std::mem::transmute::<[u8; 4], u32>([
                $data[$start + 0],
                $data[$start + 1],
                $data[$start + 2],
                $data[$start + 3],
            ])
        }
    };
    (<$start:expr, u64>, $data: expr ) => {
        unsafe {
            std::mem::transmute::<[u8; 8], u64>([
                $data[$start + 0],
                $data[$start + 1],
                $data[$start + 2],
                $data[$start + 3],
                $data[$start + 4],
                $data[$start + 5],
                $data[$start + 6],
                $data[$start + 7],
            ])
        }
    };
    (<$start:expr, $t_type:tt @ $t_temp:tt>, $data: expr) => {
        unsafe {
            std::mem::transmute::<$t_temp, $t_type>(fast_transmute!(<$start, $t_temp>, $data))
        }
    }
}