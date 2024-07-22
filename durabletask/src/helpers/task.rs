fn get_function_name<F>(_: F) -> &'static str {
    let mut name = std::any::type_name::<F>();
    if name.contains("<") {
        name = name.split("<").next().unwrap();
    }
    name.split("::").last().unwrap()
}

#[cfg(test)]
mod test {
    use crate::helpers::task::get_function_name;

    #[test]
    fn test_no_params() {
        fn test() {
            println!("hello world")
        }

        let name = get_function_name(test);

        assert_eq!(name, "test")
    }

    #[test]
    fn test_params() {
        fn test(name: String) {
            println!("hello {name}")
        }

        let name = get_function_name(test);

        assert_eq!(name, "test")
    }

    #[test]
    fn test_async_no_params() {
        async fn test() {
            println!("hello world")
        }

        let name = get_function_name(test);

        assert_eq!(name, "test")
    }

    #[test]
    fn test_async_params() {
        async fn test(name: String) {
            println!("hello {name}")
        }

        let name = get_function_name(test);

        assert_eq!(name, "test")
    }

    #[test]
    fn test_generic() {
        fn test<T>(name: T)
        where
            T: ToString,
        {
            println!("hello {}", name.to_string())
        }

        let name = get_function_name(test::<String>);

        assert_eq!(name, "test")
    }

    #[test]
    fn test_multi_generic() {
        fn test<'a, T, A>(names: T, age: A)
        where
            T: Into<Vec<&'a str>>,
            A: Into<i32>,
        {
            println!("hello {:#?} {}", names.into(), age.into())
        }

        let name = get_function_name(test::<Vec<&str>, i32>);

        assert_eq!(name, "test")
    }
}
