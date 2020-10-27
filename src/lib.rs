use std::error::Error;

const GEOHASH_BOUNDS: (f64, f64, f64, f64) = (-180.0, 180.0, -90.0, 90.0);
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
}
