fn main() {
    let args: Vec<_> = std::env::args().collect();
    let img = image::open(args[1].clone()).unwrap();
    let mut buffer = img.into_rgb8();
    let mut cnt = [0; 3];
    for pixel in buffer.pixels_mut() {
        let channel_0 = encode(pixel.0[0], cnt[0], true, false, false);
        cnt[0] = channel_0.1;
        let channel_1 = encode(pixel.0[1], cnt[1], true, false, false);
        cnt[1] = channel_1.1;
        let channel_2 = encode(pixel.0[2], cnt[2], true, false, false);
        cnt[2] = channel_2.1;
        *pixel = image::Rgb([channel_0.0 as u8, channel_1.0 as u8, channel_2.0 as u8]);
    }

    buffer.save(format!("encoded-{}", args[1])).unwrap();
}

fn encode(color: u8, cnt_prev: isize, data_enable: bool, c0: bool, c1: bool) -> (u16, isize) {
    if !data_enable {
        return match (c1, c0) {
            (false, false) => (0b0010101011, cnt_prev),
            (false, true) => (0b1101010100, cnt_prev),
            (true, false) => (0b0010101010, cnt_prev),
            (true, true) => (0b1101010101, cnt_prev),
        }
    }

    let mut q_m: u16 = 0;
    let mut q_out: u16 = 0;
    let mut cnt = cnt_prev;
    q_m = color as u16 & 0b0000_0001;
    
    if color.count_ones() > 4 || (color.count_ones() == 4 && color & 0b0000_0001 == 0) {
        for i in 0..=6 {
            let mask = 1 << i;
            q_m = !((q_m & mask) ^ (color as u16 & (mask << 1)));
        }
    } else {
        for i in 0..=6 {
            let mask = 1 << i;
            q_m = (q_m & mask) ^ (color as u16 & (mask << 1));
        }
        q_m = q_m.saturating_add(0b0000_0001_0000_0000);
    }

    if cnt_prev == 0 || (q_m as u8).count_ones() == (q_m as u8).count_zeros() {
        q_out += ((!q_m) & 0b0000_0001_0000_0000) << 1;
        q_out += q_m & 0b0000_0001_0000_0000;

        if q_m & 0b0000_0001_0000_0000 != 0 {
            q_out += (q_m as u8) as u16;
            cnt += (q_m as u8).count_ones() as isize - (q_m as u8).count_zeros() as isize;
        } else {
            q_out += (!q_m as u8) as u16;
            cnt -= (q_m as u8).count_ones() as isize - (q_m as u8).count_zeros() as isize;
        }
    } else {
        if cnt_prev > 0 && (q_m as u8).count_ones() > (q_m as u8).count_zeros()
            ||
           cnt_prev < 0 && (q_m as u8).count_ones() < (q_m as u8).count_zeros() {
            // there were more ones (or zeros) and there still are
            q_out |= 0b0000_0010_0000_0000;
            q_out |= q_m & 0b0000_0001_0000_0000;
            q_out += (!q_m as u8) as u16;
            cnt -= (q_m as u8).count_ones() as isize - (q_m as u8).count_zeros() as isize;
            cnt += ((q_m & 0b0000_0001_0000_0000) * 2) as isize;
        } else {
            q_out &= !0b0000_0010_0000_0000;
            q_out |= q_m & 0b0000_0001_0000_0000;
            q_out += (q_m as u8) as u16;
            cnt += (q_m as u8).count_ones() as isize - (q_m as u8).count_zeros() as isize;
            cnt -= (((!q_m) & 0b0000_0001_0000_0000) * 2) as isize;
        }
    }

    (q_out, cnt)
}

fn decode() {
    todo!()
}
