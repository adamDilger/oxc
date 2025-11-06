use oxc_ast::{
    AstKind,
    ast::{Expression, IdentifierName, StringLiteral},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode,
    context::LintContext,
    // fixer::{RuleFix, RuleFixer},
    rule::Rule,
};

fn component_definition_name_casing_diagnostic(span: Span) -> OxcDiagnostic {
    // See <https://oxc.rs/docs/contribute/linter/adding-rules.html#diagnostics> for details
    OxcDiagnostic::warn("Should be an imperative statement about what is wrong")
        .with_help("Should be a command-like statement that tells the user how to fix the issue")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct ComponentDefinitionNameCasing;

// See <https://github.com/oxc-project/oxc/issues/6050> for documentation details.
declare_oxc_lint!(
    /// ### What it does
    ///
    /// Briefly describe the rule's purpose.
    ///
    /// ### Why is this bad?
    ///
    /// Explain why violating this rule is problematic.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    ///export default {
    ///  /* ✗ BAD */
    ///  name: 'my-component'
    ///}
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    ///export default {
    ///  /* ✓ GOOD */
    ///  name: 'MyComponent'
    ///}
    /// ```
    ComponentDefinitionNameCasing,
    vue,
    style,
    fix, // TODO: describe fix capabilities. Remove if no fix can be done,
             // keep at 'pending' if you think one could be added but don't know how.
             // Options are 'fix', 'fix_dangerous', 'suggestion', and 'conditional_fix_suggestion'
);

impl Rule for ComponentDefinitionNameCasing {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::ObjectProperty(object_property) = node.kind() {
            if !object_property.key.is_specific_static_name("name") {
                return;
            }

            let Expression::StringLiteral(string_literal) = &object_property.value else { return };

            println!("Found component name: {}", string_literal.value);

            let identifier_name: &StringLiteral = string_literal;
            check_identifier(identifier_name, ctx);
        } else if let AstKind::CallExpression(call_expression) = node.kind() {
            let callee = &call_expression.callee;

            let Expression::StaticMemberExpression(static_member_expression) = callee else {
                return;
            };

            let IdentifierName { span, name } = &static_member_expression.property;
            if name != "component" {
                return;
            }

            println!("Found static member expression property: {:?}", span);
            println!("Found method name: {}", name);

            if call_expression.arguments.is_empty() {
                return;
            }

            let first_arg = &call_expression.arguments[0];
            let Some(Expression::StringLiteral(identifier_name)) = &first_arg.as_expression()
            else {
                return;
            };

            println!("Found first argument: {:?}", identifier_name.value);
            check_identifier(identifier_name, ctx);
        }
    }
}

fn check_identifier(identifier_name: &StringLiteral, ctx: &LintContext) {
    if has_symbols(&identifier_name.value) {
        ctx.diagnostic(component_definition_name_casing_diagnostic(identifier_name.span));
        return;
    }

    let is_kebab_case = is_kebab_case(&identifier_name.value);
    if is_kebab_case {
        ctx.diagnostic_with_fix(
            component_definition_name_casing_diagnostic(identifier_name.span),
            |fixer| {
                fixer.replace(
                    identifier_name.span.shrink(1),
                    kebab_to_pascal(&identifier_name.value),
                )
            },
        );
    }
}

fn has_symbols(str: &str) -> bool {
    for c in str.chars() {
        if !c.is_ascii_alphabetic() && c != '-' {
            return true;
        }
    }
    false
}

fn has_upper(str: &str) -> bool {
    for c in str.chars() {
        if c.is_uppercase() {
            return true;
        }
    }
    false
}

fn starts_with_hyphen(str: &str) -> bool {
    if let Some(first_char) = str.chars().next() {
        return first_char == '-';
    }
    false
}

fn has_consecutive_hyphens(str: &str) -> bool {
    let mut prev_was_hyphen = false;
    for c in str.chars() {
        if c == '-' {
            if prev_was_hyphen {
                return true;
            }
            prev_was_hyphen = true;
        } else {
            prev_was_hyphen = false;
        }
    }
    false
}

fn has_underscore_or_whitespace(str: &str) -> bool {
    for c in str.chars() {
        if c == '_' || c.is_whitespace() {
            return true;
        }
    }
    false
}

fn is_kebab_case(str: &str) -> bool {
    return !has_upper(str) &&
    !has_symbols(str) &&
    !starts_with_hyphen(str) && // starts with hyphen is not kebab-case
    !has_consecutive_hyphens(str) && // consecutive hyphens is not kebab-case
    !has_underscore_or_whitespace(str); // underscore or whitespace is not kebab-case
}

fn kebab_to_pascal(check_name: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;

    for c in check_name.chars() {
        if c == '-' {
            capitalize_next = true;
            continue;
        }

        if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }

    result
}

#[test]
fn test() {
    use crate::tester::Tester;
    use std::path::PathBuf;

    let pass = vec![
        (
            "
			        export default {
			        }
			      ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
			        export default {
			          ...name
			        }
			      ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
			        export default {
			          name: 'FooBar'
			        }
			      ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
			        export default {
			          name: 'FooBar'
			        }
			      ",
            Some(serde_json::json!(["PascalCase"])),
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
			        export default {
			          name: 'foo-bar'
			        }
			      ",
            Some(serde_json::json!(["kebab-case"])),
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        ("Vue.component('FooBar', {})", None, None, Some(PathBuf::from("test.vue"))), // languageOptions,
        (
            "Vue.component('FooBar', {})",
            Some(serde_json::json!(["PascalCase"])),
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "Vue.component('foo-bar', {})",
            Some(serde_json::json!(["kebab-case"])),
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "Vue.component(fooBar, {})",
            Some(serde_json::json!(["kebab-case"])),
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        ("Vue.component('FooBar', component)", None, None, Some(PathBuf::from("test.vue"))), // languageOptions,
        (
            "Vue.component('FooBar', component)",
            Some(serde_json::json!(["PascalCase"])),
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "Vue.component('foo-bar', component)",
            Some(serde_json::json!(["kebab-case"])),
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "Vue.component(fooBar, component)",
            Some(serde_json::json!(["kebab-case"])),
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "app.component('FooBar', component)",
            Some(serde_json::json!(["PascalCase"])),
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        ("Vue.mixin({})", None, None, Some(PathBuf::from("test.vue"))), // languageOptions,
        ("foo({})", None, None, Some(PathBuf::from("test.vue"))),       // languageOptions,
        ("foo('foo-bar', {})", None, None, Some(PathBuf::from("test.vue"))), // languageOptions,
        (
            "Vue.component(`fooBar${foo}`, component)",
            Some(serde_json::json!(["kebab-case"])),
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "app.component(`fooBar${foo}`, component)",
            Some(serde_json::json!(["kebab-case"])),
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        ("fn1(component.data)", None, None, Some(PathBuf::from("test.js"))), // languageOptions,
        ("<script setup> defineOptions({}) </script>", None, None, Some(PathBuf::from("test.vue"))), // {        "parser": require("vue-eslint-parser"),        ...languageOptions      },
        (
            "<script setup> defineOptions({name: 'FooBar'}) </script>",
            Some(serde_json::json!(["PascalCase"])),
            None,
            Some(PathBuf::from("test.vue")),
        ), // {        "parser": require("vue-eslint-parser"),        ...languageOptions      },
        (
            "<script setup> defineOptions({name: 'foo-bar'}) </script>",
            Some(serde_json::json!(["kebab-case"])),
            None,
            Some(PathBuf::from("test.vue")),
        ), // {        "parser": require("vue-eslint-parser"),        ...languageOptions      }
    ];

    let fail = vec![
        (
            "
            <script>
			        export default {
			          name: 'foo-bar'
			        }
            </script>
			      ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
            <script>
			        export default {
			          name: 'foo  bar'
			        }
            </script>
			      ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
            <script>
			        export default {
			          name: 'foo!bar'
			        }
            </script>
			      ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
			        new Vue({
			          name: 'foo!bar'
			        })
			      ",
            None,
            None,
            Some(PathBuf::from("test.js")),
        ), // { "ecmaVersion": 6 },
        (
            "
            <script>
			        export default {
			          name: 'foo_bar'
			        }
            </script>
			      ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
            <script>
			        export default {
			          name: 'foo_bar'
			        }
            </script>
			      ",
            Some(serde_json::json!(["PascalCase"])),
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
            <script>
			        export default {
			          name: 'foo_bar'
			        }
            </script>
			      ",
            Some(serde_json::json!(["kebab-case"])),
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        ("Vue.component('foo-bar', component)", None, None, Some(PathBuf::from("test.js"))), // languageOptions,
        ("app.component('foo-bar', component)", None, None, Some(PathBuf::from("test.js"))), // languageOptions,
        (
            "(Vue as VueConstructor<Vue>).component('foo-bar', component)",
            None,
            None,
            Some(PathBuf::from("test.ts")),
        ), // {        "parser": require("@typescript-eslint/parser"),        ...languageOptions      },
        ("Vue.component('foo-bar', {})", None, None, Some(PathBuf::from("test.js"))), // languageOptions,
        ("app.component('foo-bar', {})", None, None, Some(PathBuf::from("test.js"))), // languageOptions,
        (
            "Vue.component('foo_bar', {})",
            Some(serde_json::json!(["PascalCase"])),
            None,
            Some(PathBuf::from("test.js")),
        ), // languageOptions,
                                                                                      // (
                                                                                      //     "Vue.component('foo_bar', {})",
                                                                                      //     Some(serde_json::json!(["kebab-case"])),
                                                                                      //     None,
                                                                                      //     Some(PathBuf::from("test.vue")),
                                                                                      // ), // languageOptions,
                                                                                      // (
                                                                                      //     "Vue.component(`foo_bar`, {})",
                                                                                      //     Some(serde_json::json!(["kebab-case"])),
                                                                                      //     None,
                                                                                      //     Some(PathBuf::from("test.vue")),
                                                                                      // ), // languageOptions,
                                                                                      // (
                                                                                      //     "<script setup> defineOptions({name: 'foo-bar'}) </script>",
                                                                                      //     Some(serde_json::json!(["PascalCase"])),
                                                                                      //     None,
                                                                                      //     Some(PathBuf::from("test.vue")),
                                                                                      // ), // {        "parser": require("vue-eslint-parser"),        ...languageOptions      },
                                                                                      // (
                                                                                      //     "<script setup> defineOptions({name: 'FooBar'}) </script>",
                                                                                      //     Some(serde_json::json!(["kebab-case"])),
                                                                                      //     None,
                                                                                      //     Some(PathBuf::from("test.vue")),
                                                                                      // ), // {        "parser": require("vue-eslint-parser"),        ...languageOptions      }
    ];

    let fix = vec![
        (
            "
			        export default {
			          name: 'foo-bar'
			        }
			      ",
            "
			        export default {
			          name: 'FooBar'
			        }
			      ",
            None,
        ),
        // (
        //     "
        //       export default {
        //         name: 'foo_bar'
        //       }
        //     ",
        //     "
        //       export default {
        //         name: 'FooBar'
        //       }
        //     ",
        //     None,
        // ),
        // (
        //     "
        //       export default {
        //         name: 'foo_bar'
        //       }
        //     ",
        //     "
        //       export default {
        //         name: 'FooBar'
        //       }
        //     ",
        //     Some(serde_json::json!(["PascalCase"])),
        // ),
        // (
        //     "
        //       export default {
        //         name: 'foo_bar'
        //       }
        //     ",
        //     "
        //       export default {
        //         name: 'foo-bar'
        //       }
        //     ",
        //     Some(serde_json::json!(["kebab-case"])),
        // ),
        // ("Vue.component('foo-bar', component)", "Vue.component('FooBar', component)", None),
        // ("app.component('foo-bar', component)", "app.component('FooBar', component)", None),
        // (
        //     "(Vue as VueConstructor<Vue>).component('foo-bar', component)",
        //     "(Vue as VueConstructor<Vue>).component('FooBar', component)",
        //     None,
        // ),
        // ("Vue.component('foo-bar', {})", "Vue.component('FooBar', {})", None),
        // ("app.component('foo-bar', {})", "app.component('FooBar', {})", None),
        // (
        //     "Vue.component('foo_bar', {})",
        //     "Vue.component('FooBar', {})",
        //     Some(serde_json::json!(["PascalCase"])),
        // ),
        // (
        //     "Vue.component('foo_bar', {})",
        //     "Vue.component('foo-bar', {})",
        //     Some(serde_json::json!(["kebab-case"])),
        // ),
        // (
        //     "Vue.component(`foo_bar`, {})",
        //     "Vue.component(`foo-bar`, {})",
        //     Some(serde_json::json!(["kebab-case"])),
        // ),
        // (
        //     "<script setup> defineOptions({name: 'foo-bar'}) </script>",
        //     "<script setup> defineOptions({name: 'FooBar'}) </script>",
        //     Some(serde_json::json!(["PascalCase"])),
        // ),
        // (
        //     "<script setup> defineOptions({name: 'FooBar'}) </script>",
        //     "<script setup> defineOptions({name: 'foo-bar'}) </script>",
        //     Some(serde_json::json!(["kebab-case"])),
        // ),
    ];
    Tester::new(
        ComponentDefinitionNameCasing::NAME,
        ComponentDefinitionNameCasing::PLUGIN,
        pass,
        fail,
    )
    .expect_fix(fix)
    .test();
    // .test_and_snapshot();
}
