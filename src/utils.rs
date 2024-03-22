pub fn find_two_largest<T, F, K: Ord + Copy>(mut iter: impl Iterator<Item = T>, mut selector: F) -> (T, T)
where
    F: FnMut(&T) -> K,
{
    let mut largest = iter.next().unwrap();
    let mut second_largest = iter.next().unwrap();

    // Initialize based on the selector's criteria
    if selector(&second_largest) > selector(&largest) {
        std::mem::swap(&mut largest, &mut second_largest);
    }

    for item in iter {
        let key = selector(&item);
        if key > selector(&largest) {
            second_largest = largest;
            largest = item;
        } else if key > selector(&second_largest) {
            second_largest = item;
        }
    }

    (largest, second_largest)
}
