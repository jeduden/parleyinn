use super::*;

#[test]
fn a_recall_tool_is_one_of_the_six() {
    for name in [
        "event_search",
        "event_expand",
        "event_get",
        "landmark_list",
        "pin_list",
        "stat_list",
    ] {
        assert_eq!(recall_tool(Some(name)), Some(name));
    }
    assert_eq!(recall_tool(Some("mcp_server_get")), None);
    assert_eq!(recall_tool(Some("kernel_exec")), None);
    assert_eq!(recall_tool(None), None);
}

#[test]
fn each_tool_declares_the_parameters_section_9_2_specifies() {
    let tools = list()["tools"].clone();
    let params = |name: &str| -> Vec<String> {
        let tool = tools
            .as_array()
            .unwrap()
            .iter()
            .find(|t| t["name"] == name)
            .unwrap();
        let mut names: Vec<String> = tool["inputSchema"]["properties"]
            .as_object()
            .unwrap()
            .keys()
            .cloned()
            .collect();
        names.sort();
        names
    };

    assert_eq!(params("event_expand"), ["cursor", "range", "room", "scope"]);
    assert_eq!(params("event_get"), ["address", "cursor", "room", "scope"]);
    assert_eq!(params("landmark_list"), ["room", "scope", "tier"]);
    assert_eq!(params("pin_list"), ["room", "scope"]);
    assert_eq!(params("stat_list"), ["room", "scope"]);
    assert!(params("mcp_server_get").is_empty());
    assert_eq!(
        params("event_search"),
        [
            "all_of",
            "any_of",
            "k",
            "kind",
            "phrase",
            "provenance",
            "query",
            "room",
            "run",
            "scope",
            "since",
            "trust",
            "until"
        ]
    );
}
