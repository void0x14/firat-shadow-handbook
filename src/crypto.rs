//! Standalone cryptography implementations adhering to strict KISS principles.
//! Direct translations from RFC pseudocode with no generics, traits, or complex wrappers.

/// SHA-1 Implementation based on RFC 3174 Section 6.1
/// Calculates the SHA-1 hash of the given data.
pub fn sha1(data: &[u8]) -> [u8; 20] {
    // Initial context state per RFC 3174 Section 6.1
    let mut h0: u32 = 0x67452301;
    let mut h1: u32 = 0xEFCDAB89;
    let mut h2: u32 = 0x98BADCFE;
    let mut h3: u32 = 0x10325476;
    let mut h4: u32 = 0xC3D2E1F0;

    let len = data.len();
    let bit_len = (len as u64).wrapping_mul(8);

    // Padding: 1 bit (0x80) followed by 0 bits and then the 64-bit length
    // The padded message length must be a multiple of 64 bytes.
    let padding_len = 64 - ((len + 8) % 64);
    let padding_len = if padding_len == 0 { 64 } else { padding_len };
    let total_len = len + padding_len + 8;

    let mut padded_msg: Vec<u8> = Vec::with_capacity(total_len);
    padded_msg.extend_from_slice(data);
    padded_msg.push(0x80);

    // Add zeros
    for _ in 0..(padding_len - 1) {
        padded_msg.push(0);
    }

    // Add length
    padded_msg.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in padded_msg.chunks_exact(64) {
        let mut w = [0u32; 80];

        for t in 0..16 {
            w[t] = u32::from_be_bytes([
                chunk[t * 4],
                chunk[t * 4 + 1],
                chunk[t * 4 + 2],
                chunk[t * 4 + 3],
            ]);
        }

        for t in 16..80 {
            w[t] = (w[t - 3] ^ w[t - 8] ^ w[t - 14] ^ w[t - 16]).rotate_left(1);
        }

        let mut a = h0;
        let mut b = h1;
        let mut c = h2;
        let mut d = h3;
        let mut e = h4;

        for t in 0..80 {
            let (f, k) = if t < 20 {
                ((b & c) | ((!b) & d), 0x5A827999)
            } else if t < 40 {
                (b ^ c ^ d, 0x6ED9EBA1)
            } else if t < 60 {
                ((b & c) | (b & d) | (c & d), 0x8F1BBCDC)
            } else {
                (b ^ c ^ d, 0xCA62C1D6)
            };

            let temp = a
                .rotate_left(5)
                .wrapping_add(f)
                .wrapping_add(e)
                .wrapping_add(k)
                .wrapping_add(w[t]);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = temp;
        }

        h0 = h0.wrapping_add(a);
        h1 = h1.wrapping_add(b);
        h2 = h2.wrapping_add(c);
        h3 = h3.wrapping_add(d);
        h4 = h4.wrapping_add(e);
    }

    let mut out = [0u8; 20];
    out[0..4].copy_from_slice(&h0.to_be_bytes());
    out[4..8].copy_from_slice(&h1.to_be_bytes());
    out[8..12].copy_from_slice(&h2.to_be_bytes());
    out[12..16].copy_from_slice(&h3.to_be_bytes());
    out[16..20].copy_from_slice(&h4.to_be_bytes());
    out
}

/// SHA-256 Implementation based on FIPS 180-4 Section 6.2
/// Calculates the SHA-256 hash of the given data.
pub fn sha256(data: &[u8]) -> [u8; 32] {
    let mut h_state: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];

    let k: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];

    let len = data.len();
    let bit_len = (len as u64).wrapping_mul(8);

    let padding_len = 64 - ((len + 8) % 64);
    let padding_len = if padding_len == 0 { 64 } else { padding_len };
    let total_len = len + padding_len + 8;

    let mut padded_msg: Vec<u8> = Vec::with_capacity(total_len);
    padded_msg.extend_from_slice(data);
    padded_msg.push(0x80);

    for _ in 0..(padding_len - 1) {
        padded_msg.push(0);
    }

    padded_msg.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in padded_msg.chunks_exact(64) {
        let mut w = [0u32; 64];

        for t in 0..16 {
            w[t] = u32::from_be_bytes([
                chunk[t * 4],
                chunk[t * 4 + 1],
                chunk[t * 4 + 2],
                chunk[t * 4 + 3],
            ]);
        }

        for t in 16..64 {
            let s0 = w[t - 15].rotate_right(7) ^ w[t - 15].rotate_right(18) ^ (w[t - 15] >> 3);
            let s1 = w[t - 2].rotate_right(17) ^ w[t - 2].rotate_right(19) ^ (w[t - 2] >> 10);
            w[t] = w[t - 16]
                .wrapping_add(s0)
                .wrapping_add(w[t - 7])
                .wrapping_add(s1);
        }

        let mut a = h_state[0];
        let mut b = h_state[1];
        let mut c = h_state[2];
        let mut d = h_state[3];
        let mut e = h_state[4];
        let mut f = h_state[5];
        let mut g = h_state[6];
        let mut h = h_state[7];

        for t in 0..64 {
            // T1 = h + \Sigma_1(e) + Ch(e,f,g) + K_t + W_t
            let ch = (e & f) ^ ((!e) & g);
            let sigma1_e = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let t1 = h
                .wrapping_add(sigma1_e)
                .wrapping_add(ch)
                .wrapping_add(k[t])
                .wrapping_add(w[t]);

            // T2 = \Sigma_0(a) + Maj(a,b,c)
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let sigma0_a = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let t2 = sigma0_a.wrapping_add(maj);

            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }

        h_state[0] = h_state[0].wrapping_add(a);
        h_state[1] = h_state[1].wrapping_add(b);
        h_state[2] = h_state[2].wrapping_add(c);
        h_state[3] = h_state[3].wrapping_add(d);
        h_state[4] = h_state[4].wrapping_add(e);
        h_state[5] = h_state[5].wrapping_add(f);
        h_state[6] = h_state[6].wrapping_add(g);
        h_state[7] = h_state[7].wrapping_add(h);
    }

    let mut out = [0u8; 32];
    for (i, v) in h_state.iter().enumerate() {
        out[i * 4..(i + 1) * 4].copy_from_slice(&v.to_be_bytes());
    }
    out
}

/// HMAC-SHA256 Implementation based on RFC 2104 Section 2
/// Calculates the HMAC of data using SHA-256 and the given key.
pub fn hmac_sha256(key: &[u8], text: &[u8]) -> [u8; 32] {
    let b = 64; // Block size for SHA-256 (B=64 in RFC)

    // "Applications that use keys longer than B bytes will first hash the key using H
    // and then use the resultant L byte string as the actual key to HMAC."
    let mut k = [0u8; 64];
    if key.len() > b {
        k[..32].copy_from_slice(&sha256(key));
    } else {
        k[..key.len()].copy_from_slice(key);
    }

    // "(1) append zeros to the end of K to create a B byte string"
    // (Already satisfied because `k` is initialized to zeros and padding is implicit)

    let mut k_ipad = [0u8; 64];
    let mut k_opad = [0u8; 64];
    for i in 0..b {
        // "(2) XOR (bitwise exclusive-OR) the B byte string computed in step (1) with ipad"
        k_ipad[i] = k[i] ^ 0x36;
        // "(5) XOR (bitwise exclusive-OR) the B byte string computed in step (1) with opad"
        k_opad[i] = k[i] ^ 0x5c;
    }

    // "(3) append the stream of data 'text' to the B byte string resulting from step (2)"
    let mut inner_stream = Vec::with_capacity(64 + text.len());
    inner_stream.extend_from_slice(&k_ipad);
    inner_stream.extend_from_slice(text);

    // "(4) apply H to the stream generated in step (3)"
    let h_inner = sha256(&inner_stream);

    // "(6) append the H result from step (4) to the B byte string resulting from step (5)"
    let mut outer_stream = Vec::with_capacity(64 + 32);
    outer_stream.extend_from_slice(&k_opad);
    outer_stream.extend_from_slice(&h_inner);

    // "(7) apply H to the stream generated in step (6) and output the result"
    sha256(&outer_stream)
}

#[cfg(test)]
mod tests {
    use super::*;

    // RFC 3174 Section 7.3 Test Vector 1 for SHA-1
    #[test]
    fn test_sha1_rfc3174() {
        let input = b"abc";
        let expected = [
            0xa9, 0x99, 0x3e, 0x36, 0x47, 0x06, 0x81, 0x6a, 0xba, 0x3e, 0x25, 0x71, 0x78, 0x50,
            0xc2, 0x6c, 0x9c, 0xd0, 0xd8, 0x9d,
        ];
        assert_eq!(sha1(input), expected);
    }

    // FIPS 180-4 Appendix B.1 for SHA-256
    #[test]
    fn test_sha256_fips180_4() {
        let input = b"abc";
        let expected = [
            0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea, 0x41, 0x41, 0x40, 0xde, 0x5d, 0xae,
            0x22, 0x23, 0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17, 0x7a, 0x9c, 0xb4, 0x10, 0xff, 0x61,
            0xf2, 0x00, 0x15, 0xad,
        ];
        assert_eq!(sha256(input), expected);
    }

    // RFC 4231 Section 4.2 (Test Case 1 for HMAC-SHA256)
    #[test]
    fn test_hmac_sha256_rfc4231() {
        let key = [0x0b; 20];
        let data = b"Hi There";
        let expected = [
            0xb0, 0x34, 0x4c, 0x61, 0xd8, 0xdb, 0x38, 0x53, 0x5c, 0xa8, 0xaf, 0xce, 0xaf, 0x0b,
            0xf1, 0x2b, 0x88, 0x1d, 0xc2, 0x00, 0xc9, 0x83, 0x3d, 0xa7, 0x26, 0xe9, 0x37, 0x6c,
            0x2e, 0x32, 0xcf, 0xf7,
        ];
        assert_eq!(hmac_sha256(&key, data), expected);
    }
}
