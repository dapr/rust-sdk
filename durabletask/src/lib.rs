pub mod helpers;

pub mod durable_task {
    tonic::include_proto!("durabletask"); // The string specified here must match the proto package name
}
pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
