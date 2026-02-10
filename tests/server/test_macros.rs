/// Generates mechanical CRUD tests that are identical across entities.
/// Entity-specific tests remain hand-written in each file.
macro_rules! define_crud_tests {
    (
        entity: $entity:ident,
        path: $path:expr,
        list_type: $list_type:ty
        $(, malformed_json: $malformed:expr)?
        $(, missing_fields: $missing:expr)?
    ) => {
        paste::paste! {
            #[tokio::test]
            async fn [<getting_a_nonexistent_ $entity _returns_a_404>]() {
                let app = crate::helpers::spawn_app_with_auth().await;
                let client = reqwest::Client::new();

                let response = client
                    .get(app.api_url(&format!("{}/999999", $path)))
                    .send()
                    .await
                    .expect("Failed to execute request");

                assert_eq!(response.status(), 404);
            }

            #[tokio::test]
            async fn [<deleting_a_nonexistent_ $entity _returns_a_404>]() {
                let app = crate::helpers::spawn_app_with_auth().await;
                let client = reqwest::Client::new();

                let response = client
                    .delete(app.api_url(&format!("{}/999999", $path)))
                    .bearer_auth(app.auth_token.as_ref().unwrap())
                    .send()
                    .await
                    .expect("Failed to execute request");

                assert_eq!(response.status(), 404);
            }

            #[tokio::test]
            async fn [<listing_ $entity s_returns_a_200_with_empty_list>]() {
                let app = crate::helpers::spawn_app_with_auth().await;
                let client = reqwest::Client::new();

                let response = client
                    .get(app.api_url($path))
                    .send()
                    .await
                    .expect("Failed to execute request");

                assert_eq!(response.status(), 200);

                let items: Vec<$list_type> =
                    response.json().await.expect("Failed to parse response");
                assert_eq!(items.len(), 0);
            }

            $(
                #[tokio::test]
                async fn [<creating_a_ $entity _with_malformed_json_returns_a_400>]() {
                    let app = crate::helpers::spawn_app_with_auth().await;
                    let client = reqwest::Client::new();

                    let response = client
                        .post(app.api_url($path))
                        .bearer_auth(app.auth_token.as_ref().unwrap())
                        .header("content-type", "application/json")
                        .body($malformed)
                        .send()
                        .await
                        .expect("Failed to execute request");

                    assert_eq!(response.status(), 400);
                }
            )?

            $(
                #[tokio::test]
                async fn [<creating_a_ $entity _with_missing_required_fields_returns_a_400>]() {
                    let app = crate::helpers::spawn_app_with_auth().await;
                    let client = reqwest::Client::new();

                    let response = client
                        .post(app.api_url($path))
                        .bearer_auth(app.auth_token.as_ref().unwrap())
                        .header("content-type", "application/json")
                        .body($missing)
                        .send()
                        .await
                        .expect("Failed to execute request");

                    assert_eq!(response.status(), 400);
                }
            )?
        }
    };
}

pub(crate) use define_crud_tests;

/// Generates list (with/without datastar header) and delete (with datastar header)
/// tests for a given entity. The setup function creates an entity and returns its
/// ID as a String (used for the delete test; ignored for list tests).
macro_rules! define_datastar_entity_tests {
    (
        entity: $entity:ident,
        type_param: $type_param:expr,
        api_path: $api_path:expr,
        list_element: $list_element:expr,
        selector: $selector:expr,
        setup: $setup:expr
    ) => {
        paste::paste! {
            #[tokio::test]
            async fn [<$entity _list_with_datastar_header_returns_fragment>]() {
                let app = crate::helpers::spawn_app_with_auth().await;
                $setup(&app).await;
                let client = reqwest::Client::new();

                let response = client
                    .get(format!("{}/data?type={}", app.address, $type_param))
                    .header("datastar-request", "true")
                    .send()
                    .await
                    .expect(concat!("failed to fetch ", stringify!($entity)));

                assert_eq!(response.status(), 200);
                crate::helpers::assert_datastar_headers_with_mode(
                    &response,
                    "#data-content",
                    "inner",
                );

                let body = response.text().await.expect("failed to read body");
                crate::helpers::assert_html_fragment(&body);
                assert!(
                    body.contains($list_element),
                    "Fragment should contain the selector element"
                );
            }

            #[tokio::test]
            async fn [<$entity _list_without_datastar_header_returns_full_page>]() {
                let app = crate::helpers::spawn_app_with_auth().await;
                $setup(&app).await;
                let client = reqwest::Client::new();

                let response = client
                    .get(format!("{}/data?type={}", app.address, $type_param))
                    .send()
                    .await
                    .expect(concat!("failed to fetch ", stringify!($entity)));

                assert_eq!(response.status(), 200);
                assert!(response.headers().get("datastar-selector").is_none());

                let body = response.text().await.expect("failed to read body");
                crate::helpers::assert_full_page(&body);
            }

            #[tokio::test]
            async fn [<$entity _delete_with_datastar_header_returns_fragment>]() {
                let app = crate::helpers::spawn_app_with_auth().await;
                let entity_id = $setup(&app).await;
                let client = reqwest::Client::new();

                let response = client
                    .delete(app.api_url(&format!("{}/{}", $api_path, entity_id)))
                    .bearer_auth(app.auth_token.as_ref().unwrap())
                    .header("datastar-request", "true")
                    .header("referer", format!("{}/data?type={}", app.address, $type_param))
                    .send()
                    .await
                    .expect(concat!("failed to delete ", stringify!($entity)));

                assert_eq!(response.status(), 200);
                crate::helpers::assert_datastar_headers(&response, $selector);

                let body = response.text().await.expect("failed to read body");
                crate::helpers::assert_html_fragment(&body);
            }
        }
    };
}

pub(crate) use define_datastar_entity_tests;

/// Generates form submission tests for a given entity.
/// The `setup_and_form` function creates any prerequisite entities and returns
/// the form fields as `Vec<(String, String)>`.
///
/// Generated tests:
/// - `creating_{entity}_via_form_returns_redirect` — POST form → 303 redirect
/// - `creating_{entity}_via_form_with_datastar_returns_fragment` — POST form + datastar → 200
macro_rules! define_form_create_tests {
    (
        entity: $entity:ident,
        api_path: $api_path:expr,
        redirect_prefix: $redirect_prefix:expr,
        setup_and_form: $setup_fn:expr
    ) => {
        paste::paste! {
            #[tokio::test]
            async fn [<creating_ $entity _via_form_returns_redirect>]() {
                let app = crate::helpers::spawn_app_with_auth().await;
                let form_fields = $setup_fn(&app).await;
                let response = crate::helpers::post_form(&app, $api_path, &form_fields).await;

                assert_eq!(
                    response.status(),
                    303,
                    "expected 303 See Other, got {}",
                    response.status()
                );
                let location = response
                    .headers()
                    .get("location")
                    .and_then(|v| v.to_str().ok())
                    .expect("missing Location header");
                assert!(
                    location.starts_with($redirect_prefix),
                    "expected redirect to start with '{}', got '{}'",
                    $redirect_prefix,
                    location
                );
            }

            #[tokio::test]
            async fn [<creating_ $entity _via_form_with_datastar_returns_fragment>]() {
                let app = crate::helpers::spawn_app_with_auth().await;
                let form_fields = $setup_fn(&app).await;
                let response =
                    crate::helpers::post_form_datastar(&app, $api_path, &form_fields).await;

                assert_eq!(
                    response.status(),
                    200,
                    "expected 200 OK for datastar form, got {}",
                    response.status()
                );
                assert!(
                    response.headers().get("datastar-selector").is_some(),
                    "expected datastar-selector header"
                );
                assert!(
                    response.headers().get("datastar-mode").is_some(),
                    "expected datastar-mode header"
                );
            }
        }
    };
}

pub(crate) use define_form_create_tests;
