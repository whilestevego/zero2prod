use crate::test_app::TestApp;

// `tokio::test` is the testing equivalent of `tokio::main`.
// It also spares you from having to specify the `#[test]` attribute.
//
// You can inspect what code gets generated using
// `cargo expand --test health_check` (<- name of the test file)
#[tokio::test]
async fn health_check_works() {
    let app = TestApp::spawn().await;
    let response = app.get_health_check().await;

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}
