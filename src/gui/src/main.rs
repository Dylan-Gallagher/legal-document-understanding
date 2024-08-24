use article_extractor::{ArticleExtractor, LegalDocument, Relation};
use eframe::{run_native, App, CreationContext};
use egui::Context;
use egui_graphs::{
    to_graph, DefaultEdgeShape, DefaultNodeShape, Edge, Graph, GraphView, SettingsInteraction,
    SettingsStyle,
};
// use petgraph::adj::NodeIndex;
use petgraph::stable_graph::{DefaultIx, EdgeIndex, NodeIndex, StableGraph};
use petgraph::Directed;
use std::borrow::{Borrow, BorrowMut};
use std::collections::{HashMap, HashSet};

// /// We derive Deserialize/Serialize so we can persist app state on shutdown.
// #[derive(serde::Deserialize, serde::Serialize)]
// #[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct LegalComplianceApp {
    g: Graph<(), ()>,
    source_doc: LegalDocument,
    target_doc: LegalDocument,
    source_article_content: String,
    target_article_content: String,
    edge_mapping: HashMap<EdgeIndex, (NodeIndex, NodeIndex)>,
    selected_edge: Option<EdgeIndex>,
    article_extractor: ArticleExtractor,
}

impl Default for LegalComplianceApp {
    fn default() -> Self {
        let config = HashMap::from([(String::from("extraction_method"), String::from("external"))]);
        let mut article_extractor = ArticleExtractor::new(config);
        let source_doc = LegalDocument::AiAct;
        let target_doc = LegalDocument::GDPR;
        article_extractor.run(&source_doc, &target_doc);

        let g = generate_updated_graph(&article_extractor.external_references);
        // println!(
        //     "Updating graph with {:?}",
        //     &article_extractor.external_references
        // );
        let source_article_content = "Article 23
        4. For the purpose of testing in real world conditions under Article 60(2), freely-given informed consent shall be obtained from the subjects of testing prior to their participation in such testing and after their having been duly informed with concise, clear, relevant, and understandable information regarding:";
        let target_article_content = "Article 61
        2. Providers or prospective providers may conduct testing of high-risk AI systems referred to in Annex III in real world conditions at any time before the placing on the market or the putting into service of the AI system on their own or in partnership with one or more deployers or prospective deployers.";
        Self {
            g,
            source_doc,
            target_doc,
            source_article_content: String::from(source_article_content),
            target_article_content: String::from(target_article_content),
            edge_mapping: HashMap::new(),
            selected_edge: None,
            article_extractor,
        }
    }
}

impl LegalComplianceApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        // if let Some(storage) = cc.storage {
        //     return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        // }

        Default::default()
    }

    fn lorem_ipsum(&mut self, ui: &mut egui::Ui) {
        ui.with_layout(
            egui::Layout::top_down(egui::Align::LEFT).with_cross_justify(true),
            |ui| {
                ui.heading("Source Document");
                ui.label(egui::RichText::new(self.source_article_content.to_owned()));
                ui.add(egui::Separator::default().grow(8.0));
                ui.heading("Target Document");
                ui.label(egui::RichText::new(self.target_article_content.to_owned()));
            },
        );
    }

    pub fn update_graph(&mut self) {
        self.edge_mapping.clear();
        let mut g_new: StableGraph<(), ()> = StableGraph::new();

        // TODO: Improve this implementation
        let mut nodes: HashMap<String, NodeIndex> = HashMap::new();

        if let Some(relation_list) = &self.article_extractor.external_references {
            for relation in relation_list {
                let head_node: NodeIndex = if let Some(&node_index) = nodes.get(&relation.head) {
                    node_index
                } else {
                    let new_node = g_new.add_node(());
                    let index = NodeIndex::new(new_node.index());
                    nodes.insert(relation.head.clone(), index);
                    index
                };

                let tail_node: NodeIndex = if let Some(&node_index) = nodes.get(&relation.tail) {
                    node_index
                } else {
                    let new_node = g_new.add_node(());
                    let index = NodeIndex::new(new_node.index());
                    nodes.insert(relation.tail.clone(), index);
                    index
                };

                let edge_idx = g_new.add_edge(head_node, tail_node, ());
                self.edge_mapping.insert(edge_idx, (head_node, tail_node));
            }
        }

        self.g = to_graph(&g_new);
        println!("New updated graph is {:?}", self.g);
    }
}

fn generate_updated_graph(relations: &Option<Vec<Relation>>) -> Graph<(), ()> {
    let mut g_new: StableGraph<(), ()> = StableGraph::new();

    // TODO: Improve this implementation
    let mut nodes: HashMap<String, NodeIndex> = HashMap::new();

    if let Some(relation_list) = relations {
        for relation in relation_list {
            let head_node: NodeIndex = if let Some(&node_index) = nodes.get(&relation.head) {
                node_index
            } else {
                let new_node = g_new.add_node(());
                let index = NodeIndex::new(new_node.index());
                nodes.insert(relation.head.clone(), index);
                index
            };

            let tail_node: NodeIndex = if let Some(&node_index) = nodes.get(&relation.tail) {
                node_index
            } else {
                let new_node = g_new.add_node(());
                let index = NodeIndex::new(new_node.index());
                nodes.insert(relation.tail.clone(), index);
                index
            };

            g_new.add_edge(head_node, tail_node, ());
        }
    }

    to_graph(&g_new)
    // g_new
}

impl eframe::App for LegalComplianceApp {
    /// Called by the frame work to save state before shutdown.
    // fn save(&mut self, storage: &mut dyn eframe::Storage) {
    //     eframe::set_value(storage, eframe::APP_KEY, self);
    // }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Update text when edge clicked
        if !self.g.selected_edges().is_empty() {
            let edge_idx = self.g.selected_edges().first().unwrap();

            self.selected_edge = Some(*edge_idx);
            let (head_node_index, tail_node_index) = self.g.edge_endpoints(*edge_idx).unwrap();
            self.source_article_content = self.g.node(head_node_index).unwrap().label();
            self.target_article_content = self.g.node(tail_node_index).unwrap().label();
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }
                egui::widgets::global_dark_light_mode_buttons(ui);
            });
        });

        egui::SidePanel::left("left_panel")
            .resizable(false)
            .show(ctx, |ui| {
                ui.heading("Document Selection");
                egui::ComboBox::from_label("Source")
                    .selected_text(format!("{:#?}", self.source_doc))
                    .show_ui(ui, |ui| {
                        // TODO: Make dropdown display "AI Act" instead of "AiAct", maybe with `impl Debug`?
                        ui.selectable_value(
                            self.source_doc.borrow_mut(),
                            LegalDocument::AiAct,
                            "AI Act",
                        );
                        ui.selectable_value(
                            self.source_doc.borrow_mut(),
                            LegalDocument::GDPR,
                            "GDPR",
                        );
                        ui.selectable_value(
                            self.source_doc.borrow_mut(),
                            LegalDocument::DGA,
                            "DGA",
                        );
                    });

                egui::ComboBox::from_label("Target")
                    .selected_text(format!("{:#?}", self.target_doc))
                    .show_ui(ui, |ui| {
                        // TODO: Make dropdown display "AI Act" instead of "AiAct", maybe with `impl Debug`?
                        ui.selectable_value(
                            self.target_doc.borrow_mut(),
                            LegalDocument::AiAct,
                            "AI Act",
                        );
                        ui.selectable_value(
                            self.target_doc.borrow_mut(),
                            LegalDocument::GDPR,
                            "GDPR",
                        );
                        ui.selectable_value(
                            self.target_doc.borrow_mut(),
                            LegalDocument::DGA,
                            "DGA",
                        );
                    });

                if ui.button("Update Graph").clicked() {
                    self.article_extractor
                        .run(&self.source_doc, &self.target_doc);
                    self.update_graph();
                    // TODO: This doesn't update sometimes
                }
            });

        egui::SidePanel::right("right_panel")
            .resizable(true) // TODO: Find out why this is not working
            .default_width(350.0)
            .width_range(100.0..=400.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    self.lorem_ipsum(ui);
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            println!("Updating graph with {:?}", self.g);
            let interaction_settings = &SettingsInteraction::new()
                .with_dragging_enabled(true)
                // .with_node_clicking_enabled(true)
                // .with_node_selection_enabled(true)
                // .with_node_selection_multi_enabled(true)
                .with_edge_clicking_enabled(true)
                .with_edge_selection_enabled(true);
            // .with_edge_selection_multi_enabled(true);
            let style_settings = &SettingsStyle::new().with_labels_always(true);
            ui.add(
                &mut GraphView::<_, _, _, _, DefaultNodeShape, DefaultEdgeShape>::new(&mut self.g)
                    .with_styles(style_settings)
                    .with_interactions(interaction_settings),
            );
        });
    }
}

fn main() {
    let native_options = eframe::NativeOptions::default();
    run_native(
        "Legal Compliance Assistant",
        native_options,
        Box::new(|cc| Box::new(LegalComplianceApp::new(cc))),
    )
    .unwrap();
}
