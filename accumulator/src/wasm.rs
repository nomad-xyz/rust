macro_rules! export_tree {
    ($depth:literal) => {
        affix::paste! {
            mod [<internal_ $depth>] {
                use wasm_bindgen::prelude::*;

                #[wasm_bindgen]
                #[derive(Debug, Default, PartialEq)]
                #[doc = "A sparse merkle tree of depth " $depth]
                pub struct [<Tree $depth>](pub(crate) crate::Tree<$depth>);

                #[wasm_bindgen]
                #[derive(Debug, Clone, Copy, PartialEq)]
                #[doc = "A merkle proof of depth " $depth]
                pub struct [<Proof $depth>](pub(crate) crate::Proof<$depth>);


                type Internal = crate::Tree<$depth>;
                type InternalProof = crate::Proof<$depth>;

                impl From<InternalProof> for [<Proof $depth>]{
                    fn from(p: InternalProof) -> [<Proof $depth>]{
                        [<Proof $depth>](p)
                    }
                }

                impl From<Internal> for [<Tree $depth>] {
                    fn from(p: Internal) -> [<Tree $depth>] {
                        [<Tree $depth>](p)
                    }
                }

                #[wasm_bindgen]
                impl [<Tree $depth>] {
                    #[wasm_bindgen(constructor)]
                    #[doc = "Instantiate a new sparse merkle tree of depth " $depth]
                    pub fn new() -> [<Tree $depth>] {
                        Self(Default::default())
                    }

                    #[wasm_bindgen(getter)]
                    /// Get the current count of leaves in the tree
                    pub fn count(&self) -> usize {
                        self.0.count()
                    }

                    #[wasm_bindgen(js_name = "initalRoot")]
                    #[doc = "Calculate the root of an empty sparse merkle tree of depth " $depth]
                    pub fn initial_root() -> String {
                        format!("{:?}", Internal::initial_root())
                    }

                    #[wasm_bindgen]
                    /// Push a leaf to the tree. Appends it to the first unoccupied slot
                    ///
                    /// This will fail if the underlying tree is full.
                    pub fn ingest(&mut self, element: &str) -> Result<String, JsValue> {
                        let h: ethers::prelude::H256 = element
                            .parse()
                            .map_err(|e| JsValue::from(format!("Unable to parse element as H256: {}", e)))?;
                        self.0
                            .ingest(h)
                            .map(|root| format!("{:?}", root))
                            .map_err(|e| format!("Unable to ingest element: {}", e).into())
                    }

                    #[wasm_bindgen(getter)]
                    /// Retrieve the root hash of this Merkle tree.
                    pub fn root(&self) -> String {
                        format!("{:?}", self.0.root())
                    }

                    #[wasm_bindgen(getter)]
                    /// Get the tree's depth
                    pub fn depth(&self) -> usize {
                        self.0.depth()
                    }

                    #[wasm_bindgen]
                    /// Return the leaf at `index` and a Merkle proof of its inclusion.
                    ///
                    /// The Merkle proof is in "bottom-up" order, starting with a leaf node
                    /// and moving up the tree. Its length will be exactly equal to `depth`.
                    pub fn prove(&self, index: usize) -> Result<[<Proof $depth>], JsValue> {
                        self.0
                            .prove(index)
                            .map(Into::into)
                            .map_err(|e| JsValue::from(format!("Unable to get proof for index {}: {}", index, e)))
                    }

                    #[wasm_bindgen]
                    /// Verify a proof against this tree's root.
                    pub fn verify(&self, proof: [<Proof $depth>]) -> Result<(), JsValue> {
                        self.0
                            .verify(&proof.0)
                            .map_err(|e| JsValue::from(format!("Proof verification failed: {}", e)))
                    }
                }
            }
        }
    };
}

export_tree!(2);
export_tree!(4);
export_tree!(8);
export_tree!(16);
export_tree!(32);
