#[cfg(test)]
mod tests {
    use fsearch::cli::CliArgs;
    use fsearch::SearchMode;
    use clap::Parser;

    #[test]
    fn test_cli_parsing() {
        let args = CliArgs::parse_from(&["fsearch", "hello", "--mode", "both", "--regex", "--json"]);
        assert_eq!(args.query, "hello");
        assert_eq!(args.mode, "both");
        assert!(args.regex);
        assert!(args.json);

        let opts = args.into_search_options();
        assert_eq!(opts.query, "hello");
        assert_eq!(opts.mode, SearchMode::Both);
        assert!(opts.is_regex);
    }
}
