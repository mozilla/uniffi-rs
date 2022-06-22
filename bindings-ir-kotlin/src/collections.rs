use bindings_ir::Renderer;
use tera::Result;

pub fn setup_renderer(renderer: &mut Renderer) -> Result<()> {
    renderer.add_ast_templates([
        ("ListCreate", "mutableListOf<{{ inner }}>()"),
        ("ListLen", "{{ list }}.size"),
        ("ListGet", "{{ list }}[{{ index }}]"),
        ("ListSet", "{{ list }}[{{ index }}] = {{ value }}"),
        ("ListPush", "{{ list }}.add({{ value }})"),
        ("ListPop", "{{ list }}.removeLast()"),
        ("ListEmpty", "{{ list }}.clear()"),
        ("ListIterate", include_str!("templates/ListIterate.kt")),
        ("MapCreate", "mutableMapOf<{{ key }}, {{ value }}>()"),
        ("MapLen", "{{ map }}.size"),
        ("MapGet", "{{ map }}.get({{ key }})"),
        ("MapSet", "{{ map }}.put({{ key }}, {{ value }})"),
        ("MapRemove", "{{ map }}.remove({{ key }})"),
        ("MapEmpty", "{{ map }}.clear()"),
        ("MapIterate", include_str!("templates/MapIterate.kt")),
    ])
}
