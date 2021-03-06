use std::error::Error;

const GEOHASH_BOUNDS: (f64, f64, f64, f64) = (-180.0, 180.0, -90.0, 90.0);
static GEOHASH16_CHARS: &[char] = &['0', '1', '2', '3', '4',
    '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f'];
static GEOHASH32_CHARS: &[char] = &['0', '1', '2', '3', '4',
    '5', '6', '7', '8', '9', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'j',
    'k', 'm', 'n', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z'
];

const QUADTILE_BOUNDS: (f64, f64, f64, f64) = (-20037508.342789248,
    20037508.342789248, -20037508.342789248, 20037508.342789248);
static QUADTILE_CHARS: &[char] = &['2', '0', '3', '1'];

#[derive(Clone, Copy, Debug)]
pub enum Geocode {
    Geohash,
    Geohash16,
    QuadTile,
}

impl Geocode {
    pub fn decode(&self, _value: &str)
            -> Result<(f64, f64, f64, f64), Box<dyn Error>> {
        unimplemented!(); // TODO - implement
    }

    pub fn encode(&self, x: f64, y: f64, precision: usize)
            -> Result<String, Box<dyn Error>> {
        // retreive geocode specific parameters
        let (mut min_x, mut max_x, mut min_y, mut max_y,
                char_bits, codes) = match self {
            Geocode::Geohash => (GEOHASH_BOUNDS.0, GEOHASH_BOUNDS.1,
                GEOHASH_BOUNDS.2, GEOHASH_BOUNDS.3, 5, GEOHASH32_CHARS),
            Geocode::Geohash16 => (GEOHASH_BOUNDS.0, GEOHASH_BOUNDS.1,
                GEOHASH_BOUNDS.2, GEOHASH_BOUNDS.3, 4, GEOHASH16_CHARS),
            Geocode::QuadTile => (QUADTILE_BOUNDS.0, QUADTILE_BOUNDS.1,
                QUADTILE_BOUNDS.2, QUADTILE_BOUNDS.3, 2, QUADTILE_CHARS),
        };

        // check if coordinates are valid
        if x < min_x || x > max_x || y < min_y || y > max_y {
            return Err(format!("coordinate ({}, {}) is outside of geocode range ({} - {}, {} - {})", x, y, min_x, max_x, min_y, max_y).into());
        }

        // initailize instance variables
        let mut bits_total: i8 = 0;
        let mut hash_value: usize = 0;
        let mut out = String::with_capacity(precision);

        // compute geocode code
        while out.len() < precision {
            for _ in 0..char_bits {
                if bits_total % 2 == 0 {
                    // split on x value
                    let mid = (max_x + min_x) / 2f64;
                    if x > mid {
                        hash_value = (hash_value << 1) + 1usize;
                        min_x = mid;
                    } else {
                        hash_value <<= 1;
                        max_x = mid;
                    }
                } else {
                    // split on y value
                    let mid = (max_y + min_y) / 2f64;
                    if y > mid {
                        hash_value = (hash_value << 1) + 1usize;
                        min_y = mid;
                    } else {
                        hash_value <<= 1;
                        max_y = mid;
                    }
                }
                bits_total += 1;
            }

            // append character to output
            let code: char = codes[hash_value];
            out.push(code);
            hash_value = 0;
        }

        Ok(out)
    }

    pub fn get_epsg_code(&self) -> u32 {
        match self {
            Geocode::Geohash => 4326,
            Geocode::Geohash16 => 4326,
            Geocode::QuadTile => 3857,
        }
    }

    pub fn get_intervals(&self, precision: usize) -> (f64, f64) {
        match self {
            Geocode::Geohash => {
                // calculate number of bits for latitude and longitude
                let lat_bits = (2 * precision) as f64
                    + (precision as f64 / 2.0).floor();
                let long_bits = (2 * precision) as f64
                    + (precision as f64 / 2.0).ceil();

                // calculate deltas
                let lat_delta = (GEOHASH_BOUNDS.3 - GEOHASH_BOUNDS.2) /
                    2_u32.pow(lat_bits as u32) as f64;
                let long_delta = (GEOHASH_BOUNDS.1 - GEOHASH_BOUNDS.0) /
                    2_u32.pow(long_bits as u32) as f64;

                (long_delta, lat_delta)
            },
            Geocode::Geohash16 => {
                // calculate deltas
                let lat_delta = (GEOHASH_BOUNDS.3 - GEOHASH_BOUNDS.2) /
                    2_u32.pow(2 * precision as u32) as f64;
                let long_delta = (GEOHASH_BOUNDS.1 - GEOHASH_BOUNDS.0) /
                    2_u32.pow(2 * precision as u32) as f64;

                (long_delta, lat_delta)
            },
            Geocode::QuadTile => {
                // calculate delta
                let delta = (QUADTILE_BOUNDS.1 - QUADTILE_BOUNDS.0) /
                    2_u32.pow(precision as u32) as f64;

                (delta, delta)
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Geocode;

    const APPLETON_LAT_LONG: (f64, f64) = (-88.4, 44.266667);
    const APPLETON_MERCATOR: (f64, f64) = (-9840642.99, 5506802.68);
    const FORT_COLLINS_LAT_LONG: (f64, f64) = (-105.078056, 40.559167);
    const FORT_COLLINS_MERCATOR: (f64, f64) = (-11697235.69, 4947534.74);

    #[test]
    fn geohash_encode() {
        let geocode = Geocode::Geohash;

        let result = geocode.encode(
            APPLETON_LAT_LONG.0, APPLETON_LAT_LONG.1, 6);
        assert!(result.is_ok());
        assert_eq!("dpc5u6", &result.unwrap());

        let result = geocode.encode(
            FORT_COLLINS_LAT_LONG.0, FORT_COLLINS_LAT_LONG.1, 8);
        assert!(result.is_ok());
        assert_eq!("9xjq8zs6", &result.unwrap());
    }

    #[test]
    fn geohash_intervals() {
        let geocode = Geocode::Geohash;
        assert_eq!(geocode.get_intervals(1),
            (45.0, 45.0));
        assert_eq!(geocode.get_intervals(2),
            (11.25, 5.625));
        assert_eq!(geocode.get_intervals(3),
            (1.40625, 1.40625));
        assert_eq!(geocode.get_intervals(4),
            (0.3515625, 0.17578125));
        assert_eq!(geocode.get_intervals(5),
            (0.0439453125, 0.0439453125));
        assert_eq!(geocode.get_intervals(6),
            (0.010986328125, 0.0054931640625));
    }

    #[test]
    fn geohash16_encode() {
        let geocode = Geocode::Geohash16;

        let result = geocode.encode(
            APPLETON_LAT_LONG.0, APPLETON_LAT_LONG.1, 6);
        assert!(result.is_ok());
        assert_eq!("65565d", &result.unwrap());

        let result = geocode.encode(
            FORT_COLLINS_LAT_LONG.0, FORT_COLLINS_LAT_LONG.1, 8);
        assert!(result.is_ok());
        assert_eq!("4f63647f", &result.unwrap());
    }

    #[test]
    fn geohash16_intervals() {
        let geocode = Geocode::Geohash16;
        assert_eq!(geocode.get_intervals(1), (90.0, 45.0));
        assert_eq!(geocode.get_intervals(2), (22.5, 11.25));
        assert_eq!(geocode.get_intervals(3), (5.625, 2.8125));
        assert_eq!(geocode.get_intervals(4), (1.40625, 0.703125));
        assert_eq!(geocode.get_intervals(5), (0.3515625, 0.17578125));
        assert_eq!(geocode.get_intervals(6), (0.087890625, 0.0439453125));
    }

    #[test]
    fn quadtile_encode() {
        let geocode = Geocode::QuadTile;

        let result = geocode.encode(
            APPLETON_MERCATOR.0, APPLETON_MERCATOR.1, 6);
        assert!(result.is_ok());
        assert_eq!("030222", &result.unwrap());

        let result = geocode.encode(
            FORT_COLLINS_MERCATOR.0, FORT_COLLINS_MERCATOR.1, 8);
        assert!(result.is_ok());
        assert_eq!("02310101", &result.unwrap());
    }

    #[test]
    fn quadtile_intervals() {
        let geocode = Geocode::QuadTile;
        assert_eq!(geocode.get_intervals(1),
            (20037508.342789248, 20037508.342789248));
        assert_eq!(geocode.get_intervals(2),
            (10018754.171394624, 10018754.171394624));
        assert_eq!(geocode.get_intervals(3),
            (5009377.085697312, 5009377.085697312));
        assert_eq!(geocode.get_intervals(4),
            (2504688.542848656, 2504688.542848656));
        assert_eq!(geocode.get_intervals(5),
            (1252344.271424328, 1252344.271424328));
        assert_eq!(geocode.get_intervals(6),
            (626172.135712164, 626172.135712164));
    }
}
