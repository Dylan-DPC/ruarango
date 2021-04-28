use crate::{pool::RUARANGO_POOL, rand_util::rand_name};
use anyhow::Result;
use ruarango::{
    graph::{
        input::{CreateConfigBuilder, DeleteConfigBuilder, GraphMetaBuilder, ReadConfigBuilder},
        output::{GraphMeta, List},
        EdgeDefinition, EdgeDefinitionBuilder,
    },
    ArangoEither, Graph,
};

#[tokio::test]
async fn graph_list_all() -> Result<()> {
    let conn = &*RUARANGO_POOL.get()?;
    let res: ArangoEither<List> = conn.list().await?;
    assert!(res.is_right());
    let list = res.right_safe()?;
    assert!(!list.error());
    assert_eq!(*list.code(), 200);
    assert!(list.graphs().len() >= 1);

    for graph in list.graphs() {
        assert!(!graph.id().is_empty());
        assert!(!graph.key().is_empty());
        assert!(!graph.rev().is_empty());
        assert!(!graph.name().is_empty());
        assert_eq!(graph.orphan_collections().len(), 0);

        if graph.name() == "test_graph" {
            assert_eq!(graph.edge_definitions().len(), 1);
            let ed = graph.edge_definitions().get(0).unwrap();
            assert_eq!(ed.collection(), "test_edge");
            assert_eq!(ed.to().len(), 1);
            assert_eq!(ed.from().len(), 1);
        }
    }
    Ok(())
}

fn ve(vec: Vec<&str>) -> Vec<String> {
    vec.into_iter().map(str::to_string).collect()
}

fn edge_definition() -> Result<Vec<EdgeDefinition>> {
    let ed = EdgeDefinitionBuilder::default()
        .collection("test_edge")
        .from(ve(vec!["test_coll"]))
        .to(ve(vec!["test_coll"]))
        .build()?;
    Ok(vec![ed])
}

#[tokio::test]
async fn graph_create_delete() -> Result<()> {
    let conn = &*RUARANGO_POOL.get()?;
    let graph_meta = GraphMetaBuilder::default()
        .name(rand_name())
        .edge_definitions(edge_definition()?)
        .build()?;
    let config = CreateConfigBuilder::default().graph(graph_meta).build()?;
    let res: ArangoEither<GraphMeta> = conn.create(config).await?;
    assert!(res.is_right());
    let create = res.right_safe()?;
    assert!(!create.error());
    let graph_meta = create.graph();
    let name = graph_meta.name();

    let delete_config = DeleteConfigBuilder::default()
        .name(name)
        .drop_collections(true)
        .build()?;
    let res: ArangoEither<()> = conn.delete(delete_config).await?;
    assert!(res.is_right());
    Ok(())
}

#[tokio::test]
async fn graph_read() -> Result<()> {
    let conn = &*RUARANGO_POOL.get()?;
    let config = ReadConfigBuilder::default().name("test_graph").build()?;
    let res: ArangoEither<GraphMeta> = conn.read(config).await?;
    assert!(res.is_right());
    let graph_meta = res.right_safe()?;
    assert!(!graph_meta.error());
    assert_eq!(*graph_meta.code(), 200);
    let graph = graph_meta.graph();

    assert!(!graph.id().is_empty());
    assert!(!graph.key().is_empty());
    assert!(!graph.rev().is_empty());
    assert_eq!(graph.name(), "test_graph");
    assert_eq!(graph.orphan_collections().len(), 0);
    assert_eq!(graph.edge_definitions().len(), 1);
    let ed = graph.edge_definitions().get(0).unwrap();
    assert_eq!(ed.collection(), "test_edge");
    assert_eq!(ed.to().len(), 1);
    assert_eq!(ed.from().len(), 1);

    Ok(())
}