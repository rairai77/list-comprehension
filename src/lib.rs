#[cfg(test)]
mod tests {
    use comp_macro::comp;
    #[test]
    fn it_works() {
        let result = comp![x + 1 for x in [1,2,3] if x!=3 if x!=2].collect::<Vec<_>>();
        assert_eq!(result, vec![2]);
    }
}
