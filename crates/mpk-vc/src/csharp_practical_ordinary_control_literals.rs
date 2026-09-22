//! Exact literal results at native source nodes. These local relations do not
//! establish that execution reaches the node or select an incoming edge.
use super::*;
use crate::csharp_practical_vir_validation::PracticalVirLiteral;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryControlNativeLiteral {
    pub source: PracticalVirLiteral,
    /// The result exists at its literal-producing block, including literals
    /// retained on an invocation block; it is not an invocation return value.
    pub result: ControlBinding,
    pub literal_definition: String,
    /// Complete physical storage equality, including padding and inactive arms.
    pub definition: String,
}

pub(super) fn append(
    c: &mut Clauses<'_>,
    p: &mut OrdinaryControlEdgeProgram,
    vir: &ValidatedPracticalVir,
    layouts: &OrdinaryCarrierProgram,
) -> R<()> {
    let mut values = BTreeMap::new();
    for flow in &mut p.functions {
        let native = vir
            .functions()
            .iter()
            .find(|f| f.id == flow.source.function_id)
            .ok_or(OrdinaryCarrierError::Linkage)?;
        for block in &native.blocks {
            for source in &block.literal_values {
                if source.result.type_id != source.value.type_id() {
                    return Err(OrdinaryCarrierError::Linkage);
                }
                let literal_definition = name("NativeLiteralValue", &source.value);
                values.insert(literal_definition.clone(), source.value.clone());
                let result = ControlBinding {
                    kind: "literal_result".into(),
                    edge_id: None,
                    node_id: block.node.id.clone(),
                    value_id: source.result.id.clone(),
                    type_id: source.result.type_id.clone(),
                };
                let definition = name("NativeLiteralResult", &(&native.id, &result, source));
                flow.native_literals.push(OrdinaryControlNativeLiteral {
                    source: source.clone(),
                    result,
                    literal_definition,
                    definition,
                });
            }
        }
    }
    if values.is_empty() {
        return Ok(());
    }
    let builder = std::mem::replace(&mut c.b, Builder::new()?);
    (c.b, _) =
        literals::emit_named_values_scoped(vir, layouts, builder, values, "ControlLiteralPart")?;
    for literal in p.functions.iter().flat_map(|f| &f.native_literals) {
        let depth = c
            .carriers
            .get(literal.result.type_id.as_str())
            .ok_or(OrdinaryCarrierError::Linkage)?
            .depth;
        let equality = construction_data::physical_equal(&mut c.b, depth)?;
        let actual = c.b.var(0)?;
        let expected = c.b.constant(&literal.literal_definition)?;
        let body = call(&mut c.b, &equality, vec![actual, expected])?;
        define(&mut c.b, &literal.definition, &[depth], 0, body)?;
    }
    Ok(())
}
