fn main() {
    let args: Vec<_> = std::env::args().collect();

    let filename = &String::from("pack.jpg");
    let path = args.get(1).unwrap_or(filename);

    let img = image::open(path.clone()).unwrap();
    
    let mut buffer = img.into_rgb8();
    let mut cnt = [0; 3];

    let mut channel_0_stream = Vec::new();
    let mut channel_1_stream = Vec::new();
    let mut channel_2_stream = Vec::new();

    for pixel in buffer.pixels_mut() {
        // encode
        let (channel_0, count_0) = encode(pixel.0[0], cnt[0], true, false, false);
        cnt[0] = count_0;
        
        let (channel_1, count_1) = encode(pixel.0[1], cnt[1], true, false, false);
        cnt[1] = count_1;
        
        let (channel_2, count_2) = encode(pixel.0[2], cnt[2], true, false, false);
        cnt[2] = count_2;

        channel_0_stream.extend(bits_to_booleans(channel_0));
        channel_1_stream.extend(bits_to_booleans(channel_1));
        channel_2_stream.extend(bits_to_booleans(channel_2));
    }

    // for frame in 0..500 {
    //     channel_0_stream.rotate_left(1);
    //     channel_1_stream.rotate_left(1);
    //     channel_2_stream.rotate_left(1);
    for (i, pixel) in buffer.pixels_mut().enumerate() {
        let channel_0 = booleans_to_bits(&channel_0_stream[(i*10)..][..10]);
        let channel_1 = booleans_to_bits(&channel_1_stream[(i*10)..][..10]);
        let channel_2 = booleans_to_bits(&channel_2_stream[(i*10)..][..10]);

        // decode
        let (channel_0, _, _) = decode(channel_0, true);
        let (channel_1, _, _) = decode(channel_1, true);
        let (channel_2, _, _) = decode(channel_2, true);
        
        *pixel = image::Rgb([channel_0 as u8, channel_1 as u8, channel_2 as u8]);
    }

    buffer.save(format!("encoded-decoded-frame-{path}")).unwrap();
    // buffer.save(format!("encoded-decoded-frame{frame}-{path}")).unwrap();
    // println!("frame {frame} done");
    // }
}
 
fn bits_to_booleans(ten_bits: u16) -> Vec<bool> {
    let mut bools = Vec::new();

    for i in 0..10 {
        bools.push((ten_bits & (1 << i)) != 0);
    }

    bools
}

fn booleans_to_bits(bools: &[bool]) -> u16 {
    assert!(bools.len() == 10);

    let mut ten_bits: u16 = 0;

    for bool_val in bools.iter().rev() {
        if *bool_val {
            ten_bits += 1;
        }
        
        ten_bits = ten_bits << 1;
    }
    
    ten_bits = ten_bits >> 1;

    ten_bits
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
    // 0111_0010
    if color.count_ones() > 4 || (color.count_ones() == 4 && color & 0b0000_0001 == 0) {
        for i in 0..=6 {
            let mask = 1 << i;
            q_m += (!(((q_m & mask) << 1) ^ (color as u16 & (mask << 1)))) & (mask << 1);
        }
    } else {
        for i in 0..=6 {
            let mask = 1 << i;
            q_m += (((q_m & mask) << 1) ^ (color as u16 & (mask << 1))) & (mask << 1);
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

fn decode(q_out: u16, data_enable: bool) -> (u8, bool, bool) {
    if !data_enable {
        return match q_out {
            0b0010101011 => (0, false, false),
            0b1101010100 => (0, false, true),
            0b0010101010 => (0, true, false),
            0b1101010101 => (0, true, true),
            _            => (0b0101_0101, false, false) // this shouldn't happen (but it will because we want it to break)
        }
    }

    let d9 = q_out & 0b10_0000_0000 != 0;
    let d8 = q_out & 0b01_0000_0000 != 0;
    let mut color = q_out as u8;

    if d9 {
        color = !color;
    }

    let mut new_color = color & 0b0000_0001;
    if d8 {
        for i in 0..=6 {
            let mask = 1 << i;
            new_color += ((((color & mask) << 1) ^ (color & (mask << 1)))) & (mask << 1);
        }
    } else {
        for i in 0..=6 {
            let mask = 1 << i;
            new_color += (!((color & mask) << 1) ^ (color & (mask << 1))) & (mask << 1);
        }
    }

    (new_color, false, false)
}
