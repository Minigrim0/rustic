/// Splits a slice of f32 values into chunks of a specified size, padding with zeros if necessary.
pub fn chunks_with_padding(
    data: &[f32],
    chunk_size: usize,
) -> impl Iterator<Item = (usize, Vec<f32>)> + '_ {
    data.chunks(chunk_size).enumerate().map(move |(i, chunk)| {
        let index = i * chunk_size;
        let mut padded_chunk = chunk.to_vec();

        // Pad with zeros if the chunk is smaller than chunk_size
        if padded_chunk.len() < chunk_size {
            padded_chunk.resize(chunk_size, 0.0);
        }

        (index, padded_chunk)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunks_with_padding() {
        let data = [1.0, 2.0, 3.0, 4.0, 5.0];
        let results: Vec<_> = chunks_with_padding(&data, 3).collect();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0], (0, vec![1.0, 2.0, 3.0]));
        assert_eq!(results[1], (3, vec![4.0, 5.0, 0.0])); // Padded with zero
    }

    #[test]
    fn test_exact_chunks() {
        let data = [1.0, 2.0, 3.0, 4.0];
        let results: Vec<_> = chunks_with_padding(&data, 2).collect();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0], (0, vec![1.0, 2.0]));
        assert_eq!(results[1], (2, vec![3.0, 4.0]));
    }
}
