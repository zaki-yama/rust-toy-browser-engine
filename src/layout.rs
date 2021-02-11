use std::{alloc::Layout, task::Context};

use crate::{
    css::Stylesheet,
    css::{
        Unit,
        Value::{Keyword, Length},
    },
    style::StyledNode,
};

///! Basic CSS block layout.

// CSS box model. All sizes are in px.

#[derive(Clone, Copy, Default, Debug)]
struct Dimensions {
    // Position of the content area relative to the document origin:
    content: Rect,

    // Surrounding edges:
    padding: EdgeSizes,
    border: EdgeSizes,
    margin: EdgeSizes,
}

#[derive(Clone, Copy, Default, Debug)]
struct Rect {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

#[derive(Clone, Copy, Default, Debug)]
struct EdgeSizes {
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
}

struct LayoutBox<'a> {
    dimensions: Dimensions,
    pub box_type: BoxType<'a>,
    children: Vec<LayoutBox<'a>>,
}

enum BoxType<'a> {
    BlockNode(&'a StyledNode<'a>),
    InlineNode(&'a StyledNode<'a>),
    AnonymousBlock,
}

impl<'a> LayoutBox<'a> {
    fn new(box_type: BoxType) -> LayoutBox {
        LayoutBox {
            box_type,
            dimensions: Default::default(), // initially set all fields to 0.0
            children: vec![],
        }
    }

    /// Where a new inline child should go.
    fn get_inline_container(&mut self) -> &mut LayoutBox<'a> {
        match self.box_type {
            BoxType::InlineNode(_) | BoxType::AnonymousBlock => self,
            BoxType::BlockNode(_) => {
                // If we've just generated an anonymous block box, keep using it.
                // Otherwise, create a new one.
                match self.children.last() {
                    Some(&LayoutBox {
                        box_type: BoxType::AnonymousBlock,
                        ..
                    }) => {}
                    _ => self.children.push(LayoutBox::new(BoxType::AnonymousBlock)),
                }
                self.children.last_mut().unwrap()
            }
        }
    }

    /// Lay out a box and its descendants.
    fn layout(&mut self, containing_block: Dimensions) {
        match self.box_type {
            BoxType::BlockNode(_) => self.layout_block(containing_block),
            BoxType::InlineNode(_) => {}  // TODO
            BoxType::AnonymousBlock => {} // TODO
        }
    }

    fn layout_block(&mut self, containing_block: Dimensions) {
        // Child width can depend on parent width, so we need to calculate
        // this box's width before laying out its children.
        self.calculate_block_width(containing_block);

        // Determine where the box is located within its container.
        self.calculate_block_position(containing_block);

        // Recursively lay out the children of this box.
        self.layout_block_children();

        // Parent height can depend on child height, so `calculate_height`
        // must be called *after* the children are laid out.
        self.calculate_block_height();
    }

    fn calculate_block_width(&mut self, containing_block: Dimensions) {
        let style = self.get_style_node();

        // `width` has initial value `auto`.
        let auto = Keyword(String::from("auto"));
        let mut width = style.value("width").unwrap_or(auto.clone());

        // margin, border, and padding have initial value 0.
        let zero = Length(0.0, Unit::Px);

        let mut margin_left = style.lookup("margin-left", "margin", &zero);
        let mut margin_right = style.lookup("margin-right", "margin", &zero);

        let border_left = style.lookup("border-left-width", "border-width", &zero);
        let border_right = style.lookup("border-right-width", "border-width", &zero);

        let padding_left = style.lookup("padding-left", "padding", &zero);
        let padding_right = style.lookup("padding-right", "padding", &zero);

        let total = sum([
            &margin_left,
            &margin_right,
            &border_left,
            &border_right,
            &padding_left,
            &padding_right,
            &width,
        ]
        .iter()
        .map(|v| v.to_px()));

        // If width is not auto and the total is wider than the container, treat auto margins as 0.
        if width != auto && total > containing_block.content.width {
            if margin_left == auto {
                margin_left = Length(0.0, Unit::Px);
            }
            if margin_right == auto {
                margin_right = Length(0.0, Unit::Px);
            }
        }

        // Adjust used values so that the above sum equals `containing_block.width`.
        // Each arm of the `match` should increase the total width by exactly `underflow`,
        // and afterward all values should be absolute lengths in px.
        let underflow = containing_block.content.width - total;

        match (width == auto, margin_left == auto, margin_right == auto) {
            // If the values are overconstrained, calculate margin_right.
            (false, false, false) => {
                margin_right = Length(margin_right.to_px() + underflow, Unit::Px);
            }

            // If exactly one size is auto, its used value follows from the equality.
            (false, false, true) => {
                margin_right = Length(underflow, Unit::Px);
            }
            (false, true, false) => {
                margin_left = Length(underflow, Unit::Px);
            }

            // If width is set to auto, any other auto values become 0.
            (true, _, _) => {
                if margin_left == auto {
                    margin_left = Length(0.0, Unit::Px);
                }
                if margin_right == auto {
                    margin_right = Length(0.0, Unit::Px);
                }

                if underflow >= 0.0 {
                    // Expand width to fill the underflow.
                    width = Length(underflow, Unit::Px);
                } else {
                    // Width can't be negative. Adjust the right margin instead.
                    width = Length(0.0, Unit::Px);
                    margin_right = Length(margin_right.to_px() + underflow, Unit::Px);
                }
            }

            // If margin-left and margin-right are both auto, their used values are equal.
            (false, true, true) => {
                margin_left = Length(underflow / 2.0, Unit::Px);
                margin_right = Length(underflow / 2.0, Unit::Px);
            }
        }

        let d = &mut self.dimensions;
        d.content.width = width.to_px();

        d.padding.left = padding_left.to_px();
        d.padding.right = padding_right.to_px();

        d.border.left = border_left.to_px();
        d.border.right = border_right.to_px();

        d.margin.left = margin_left.to_px();
        d.margin.right = margin_right.to_px();
    }

    fn calculate_block_position(&mut self, containing_block: Dimensions) {
        let style = self.get_style_node();
        let d = &mut self.dimensions;

        // margin, border, and padding have initial value 0.
        let zero = Length(0.0, Unit::Px);

        // If margin-top or margin-bottom is `auto`, the used value is zero.
        d.margin.top = style.lookup("margin-top", "margin", &zero).to_px();
        d.margin.bottom = style.lookup("margin-bottom", "margin", &zero).to_px();

        d.border.top = style
            .lookup("border-top-width", "border-width", &zero)
            .to_px();
        d.border.bottom = style
            .lookup("border-bottom-width", "border-width", &zero)
            .to_px();

        d.padding.top = style.lookup("padding-top", "padding", &zero).to_px();
        d.padding.bottom = style.lookup("padding-bottom", "padding", &zero).to_px();

        d.content.x = containing_block.content.x + d.margin.left + d.border.left + d.padding.left;

        // Position the box below all the previous boxes in the container.
        d.content.y = containing_block.content.height
            + containing_block.content.y
            + d.margin.top
            + d.border.top
            + d.padding.top;
    }

    fn layout_block_children(&mut self) {
        let d = &mut self.dimensions;
        for child in &mut self.children {
            child.layout(*d);
            // Track the height so each child is laid out below the previous content.
            d.content.height = d.content.height + child.dimensions.margin_box().height;
        }
    }

    fn calculate_block_height(&mut self) {
        // If the height is set to an explicit length, use that exact length.
        // Otherwise, just keep the value set by `layout_block_children`.
        if let Some(Length(h, Unit::Px)) = self.get_style_node().value("height") {
            self.dimensions.content.height = h;
        }
    }

    fn get_style_node(&self) -> &'a StyledNode<'a> {
        match self.box_type {
            BoxType::BlockNode(node) | BoxType::InlineNode(node) => node,
            BoxType::AnonymousBlock => panic!("Anonymous block box has no style node"),
        }
    }
}

/// Build the tree of LayoutBoxes, but don't perform any layout calculations yet.
fn build_layout_tree<'a>(style_node: &'a StyledNode<'a>) -> LayoutBox<'a> {
    // Create the root box.
    let mut root = LayoutBox::new(match style_node.display() {
        Block => BoxType::BlockNode(style_node),
        Inline => BoxType::InlineNode(style_node),
        DisplayNone => panic!("Root node has display: none."),
    });

    // Create the descendant boxes.
    for child in &style_node.children {
        match child.display() {
            Block => root.children.push(build_layout_tree(child)),
            Inline => root
                .get_inline_container()
                .children
                .push(build_layout_tree(child)),
            DisplayNone => {} // Skip nodess with `display: none;`
        }
    }
    root
}

impl Dimensions {
    /// The area covered by the content area plus its padding.
    fn padding_box(self) -> Rect {
        self.content.expanded_by(self.padding)
    }

    /// The area covered by the content area plus padding and borders.
    fn border_box(self) -> Rect {
        self.padding_box().expanded_by(self.border)
    }

    /// The area covered by the content area plus padding, borders and margin.
    fn margin_box(self) -> Rect {
        self.border_box().expanded_by(self.margin)
    }
}

impl Rect {
    fn expanded_by(self, edge: EdgeSizes) -> Rect {
        Rect {
            x: self.x - edge.left,
            y: self.y - edge.top,
            width: self.width + edge.left + edge.right,
            height: self.height + edge.top + edge.right,
        }
    }
}

fn sum<I>(iter: I) -> f32
where
    I: Iterator<Item = f32>,
{
    iter.fold(0., |a, b| a + b)
}
