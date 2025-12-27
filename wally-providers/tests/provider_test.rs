use paste::paste;
use tempfile::tempdir;
use wally_config::{
    util::KdlBool,
    wallhaven::{WallhavenCategories, WallhavenConfig},
};
use wally_providers::providers::{
    WallpaperProvider, konachan::Konachan, pixiv::Pixiv, wallhaven::Wallhaven,
};

macro_rules! test_provider {
    ($test_name:ident, $provider:block) => {
        paste! {
            #[ignore]
            #[tokio::test]
            async fn [<test_list_wallpapers_ $test_name>]() {
                let provider = $provider;
                let limit = 50;
                let list = provider.list(limit).await;
                assert!(list.is_ok(), "{:?}", list);
                assert!(list.unwrap().len() == limit as usize);
            }

            #[ignore]
            #[tokio::test]
            async fn [<test_get_random_wallpaper_ $test_name>]() {
                let provider = $provider;
                let url = provider.random().await;
                assert!(url.is_ok(), "{:?}", url);
            }

            #[ignore]
            #[tokio::test]
            async fn [<test_download_ $test_name>]() {
                let provider = $provider;
                let source = provider.random().await.unwrap();
                let dir = tempdir().expect("Should create a tempdir");
                let filepath = provider.download(&source, dir.path()).await.unwrap();
                assert!(filepath.exists());
            }

        }
    };
}

test_provider!(pixiv, { Pixiv::new() });

test_provider!(wallhaven, {
    Wallhaven::new(WallhavenConfig {
        categories: WallhavenCategories {
            general: KdlBool { value: true },
            anime: KdlBool { value: true },
            people: KdlBool { value: true },
        },
    })
});

test_provider!(konachan, { Konachan::new() });
