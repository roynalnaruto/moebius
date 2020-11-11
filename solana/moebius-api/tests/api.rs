mod api_tests {
    use moebius_api::MoebiusApi;

    #[test]
    fn test_uniswap_oracle() {
        let moebius_api = MoebiusApi::new();

        let result = moebius_api.uniswap_oracle(
            "1f9840a85d5af5bf1d1762f925bdaddc4201f984",
            "c778417e063141139fce010982780140aa0cd5ab",
        );

        assert!(result.is_ok());

        let pricefeed = result.unwrap();
        assert_eq!(
            pricefeed.price_token0_token1(),
            1.0f64 / pricefeed.price_token1_token0()
        );
    }
}
