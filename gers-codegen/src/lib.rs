use proc_macro::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::{parse_macro_input, punctuated::Punctuated, spanned::Spanned, Expr, Token};

/// Define a new type wrapping a vector which can be used from Wren.
///
/// Wren's lists are dynamically typed containers pointing to values
/// allocated on the heap.
///
/// To allow for optionally more performant code, and better spatial
/// locality, statically typed dynamic arrays are an alternative.
#[proc_macro]
pub fn impl_array(args: TokenStream) -> TokenStream {
    let parser = Punctuated::<Expr, Token![,]>::parse_terminated;
    let ast = parse_macro_input!(args with parser);

    if ast.is_empty() {
        return TokenStream::from(
            syn::Error::new_spanned(ast, "Expected macro arguments are empty").to_compile_error(),
        );
    }

    let prefix = match ast.first().unwrap() {
        Expr::Path(path_expr) => match path_expr.path.get_ident() {
            Some(ident) => ident.clone(),
            None => {
                return TokenStream::from(
                    syn::Error::new_spanned(path_expr, "First argument must be an identifier and not a path.")
                        .to_compile_error(),
                );
            }
        },
        arg => {
            return TokenStream::from(
                syn::Error::new_spanned(arg, "First argument must be an identifier").to_compile_error(),
            );
        }
    };

    let ty_path = match ast.iter().nth(1) {
        Some(expr) => match expr {
            Expr::Path(path_expr) => path_expr.path.clone(),
            _ => {
                return TokenStream::from(
                    syn::Error::new_spanned(expr, "Second argument must be a type").to_compile_error(),
                );
            }
        },
        None => {
            return TokenStream::from(
                syn::Error::new_spanned(ast.clone(), "Second argument missing").to_compile_error(),
            );
        }
    };

    let array_ident = format_ident!("{}Array", prefix);
    let array_ident_string = array_ident.to_string();

    // Imports are kept in a module to prevent polluting
    // the module using this macro.
    let mod_ident = format_ident!("__{}_array", prefix);

    // Trick to give the user a better error when the type does
    // not implement Clone.
    let ty_span = ty_path.span();
    let assert_clone = quote_spanned! {ty_span=>
        struct __Assert where #ty_path: Clone;
    };

    let gen = quote! {
        #[allow(non_snake_case)]
        mod #mod_ident {
            #assert_clone

            use super::*;
            use rust_wren::prelude::*;
            use crate::collections::{OutOfBounds, ArrayIterator};
            use std::fmt;

            /// Statically typed vector intended to be used in Wren scripts.
            ///
            /// For Rust just use a regular `Vec<T>`.
            ///
            /// Generated by macro `impl_array`.
            #[wren_class]
            pub struct #array_ident (Vec<#ty_path>);

            /// Associated function exposed to Wren as class methods.
            #[wren_methods]
            impl #array_ident {
                /// Creates a new empty array instance.
                ///
                /// Called by Wren as the class' constructor.
                #[construct]
                pub fn new() -> Self {
                    Self(Vec::new())
                }

                /// Retrieve the element at the given index.
                // TODO: Support subscript operator
                #[inline]
                pub fn get(&self, index: i32) -> rust_wren::Result<#ty_path> {
                    self.0.get(self.convert_index(index)).cloned().ok_or_else(|| {
                        foreign_error!(OutOfBounds {
                            index,
                            size: self.0.len()
                        })
                    })
                }

                /// Adds the given element to the end of the array.
                #[inline]
                pub fn add(&mut self, value: #ty_path) {
                    self.0.push(value)
                }

                /// Inserts the given element at the index position.
                #[inline]
                pub fn insert(&mut self, index: i32, value: #ty_path) {
                    self.0.insert(self.convert_index(index), value);
                }

                /// Removes the element at the given index and returns it.
                ///
                /// # Errors
                ///
                /// Returns a foreign error when the index is out of bounds.
                #[inline]
                #[method(name = removeAt)]
                pub fn remove_at(&mut self, index: i32) -> rust_wren::Result<#ty_path> {
                    if !self.in_bounds(index) {
                        Err(foreign_error!(OutOfBounds {
                            index,
                            size: self.0.len()
                        }))
                    } else {
                        // Vec panics on out of bounds remove.
                        Ok(self.0.remove(self.convert_index(index)))
                    }
                }

                /// Clears the contents of the array.
                ///
                /// Underlying `Vec<T>` implementation does not deallocate.
                #[inline]
                pub fn clear(&mut self) {
                    self.0.clear();
                }

                #[inline]
                pub fn iterate(&self, index: Option<i32>) -> ArrayIterator {
                    match index {
                        None => ArrayIterator::Index(0),
                        Some(index) => {
                            if index < (self.0.len() as i32) - 1 {
                                ArrayIterator::Index(index + 1)
                            } else {
                                ArrayIterator::Done
                            }
                        }
                    }
                }

                #[inline]
                #[method(name = iteratorValue)]
                pub fn iterator_value(&self, index: i32) -> rust_wren::Result<#ty_path> {
                    self.get(index)
                }
            }

            /// Associated functions not exposed to Wren.
            impl #array_ident {
                /// Convert a Wren index, which can be negative, to an
                /// unsigned Rust index.
                ///
                /// Wren allows indexing from the back of a list.
                #[inline(always)]
                fn convert_index(&self, index: i32) -> usize {
                    if index >= 0 {
                        index as usize
                    } else {
                        self.0.len() - index as usize
                    }
                }

                /// Borrows the contents of the array as an immutable slice.
                #[inline]
                pub fn as_slice(&self) -> &[#ty_path] {
                    &self.0
                }

                /// Borrows the contents of the array as an mutable slice.
                #[inline]
                pub fn as_slice_mut(&mut self) -> &mut [#ty_path] {
                    &mut self.0
                }

                /// Checks whether the given index is in range.
                ///
                /// Wren allows negative indices to access a
                /// collection from the back.
                #[inline(always)]
                fn in_bounds(&self, index: i32) -> bool {
                    index >= -(self.0.len() as i32) && index < self.0.len() as i32
                }

                /// Builds a representation of the Wren class as
                /// a script string.
                ///
                /// Can be interpreted to declare the class in a VM.
                pub fn script() -> String {
                    format!(r#"
                    foreign class {} is Sequence {{
                      construct new() {{}}

                      // TODO: Support subscript operator
                      foreign get(index)
                      foreign add(value)
                      foreign insert(index, value)
                      foreign removeAt(index)
                      foreign clear()
                      // TODO: count
                      // TODO: toString
                      foreign iterate(iterator)
                      foreign iteratorValue(iterator)
                    }}
                    "#, #array_ident_string)
                }
            }

            impl Default for #array_ident {
                fn default() -> Self {
                    Self::new()
                }
            }

            impl Into<Vec<#ty_path>> for #array_ident {
                fn into(self) -> Vec<#ty_path> {
                    self.0
                }
            }

            impl From<Vec<#ty_path>> for #array_ident {
                fn from(vector: Vec<#ty_path>) -> Self {
                    #array_ident(vector)
                }
            }

            impl fmt::Debug for #array_ident {
                #[inline]
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    f.debug_list().entries(self.0.iter()).finish()
                }
            }
        }
        // Re-export without baggage.
        pub use #mod_ident::#array_ident;
    };

    gen.into()
}
