/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use bindings_ir::Renderer;
use tera::Result;

pub fn setup_renderer(renderer: &mut Renderer) -> Result<()> {
    renderer.add_ast_templates([
        ("ClassDef", include_str!("templates/ClassDef.kt")),
        ("DataClassDef", include_str!("templates/DataClassDef.kt")),
        ("EnumDef", include_str!("templates/EnumDef.kt")),
        (
            "ExceptionBaseDef",
            include_str!("templates/ExceptionBaseDef.kt"),
        ),
        ("ExceptionDef", include_str!("templates/ExceptionDef.kt")),
        ("ClassCreate", include_str!("templates/ClassCreate.kt")),
        ("DataClassCreate", "{{ name }}({{ values|comma_join }})"),
        ("EnumCreate", include_str!("templates/EnumCreate.kt")),
        ("ExceptionCreate", "{{ name }}({{ values|comma_join }})"),
        (
            "MethodCall",
            "{{ obj }}.{{ name }}({{ values|comma_join }})",
        ),
        (
            "StaticMethodCall",
            "{{ class }}.{{ name }}({{ values|comma_join }})",
        ),
        ("IntoRust", "{{ obj }}.intoRust()"),
    ])
}
