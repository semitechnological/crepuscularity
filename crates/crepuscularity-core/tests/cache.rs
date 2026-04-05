#[cfg(not(target_arch = "wasm32"))]
mod cache_tests {
    use crepuscularity_core::analysis::Fingerprint;
    use crepuscularity_core::cache::DriverCache;
    use tempfile::TempDir;

    #[test]
    fn cache_miss_then_hit() {
        let tmp = TempDir::new().unwrap();
        let cache = DriverCache::open(tmp.path());
        let fp = Fingerprint::new("source content", None, "render");
        let output = "<div>hello</div>";

        assert!(
            !cache.is_up_to_date(&fp, output),
            "should miss on first call"
        );
        cache.record(&fp, output);
        assert!(cache.is_up_to_date(&fp, output), "should hit after record");
    }

    #[test]
    fn cache_misses_on_changed_output() {
        let tmp = TempDir::new().unwrap();
        let cache = DriverCache::open(tmp.path());
        let fp = Fingerprint::new("same source", None, "render");
        cache.record(&fp, "<div>v1</div>");
        assert!(!cache.is_up_to_date(&fp, "<div>v2</div>"));
    }

    #[test]
    fn different_fingerprints_are_independent() {
        let tmp = TempDir::new().unwrap();
        let cache = DriverCache::open(tmp.path());
        let fp1 = Fingerprint::new("source-a", None, "render");
        let fp2 = Fingerprint::new("source-b", None, "render");
        cache.record(&fp1, "output-a");
        assert!(!cache.is_up_to_date(&fp2, "output-a"));
    }
}
