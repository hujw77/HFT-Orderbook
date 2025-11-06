//! AVL Tree implementation for maintaining price levels in sorted order

use crate::types::Price;

/// AVL Tree node indices and operations
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AvlNode {
    /// Index of parent node (None for root)
    pub parent: Option<usize>,
    /// Index of left child
    pub left_child: Option<usize>,
    /// Index of right child  
    pub right_child: Option<usize>,
    /// Height of this subtree
    pub height: i32,
}

impl AvlNode {
    /// Create a new AVL node
    pub fn new() -> Self {
        Self {
            parent: None,
            left_child: None,
            right_child: None,
            height: 1,
        }
    }

    /// Check if this node is a leaf (no children)
    pub fn is_leaf(&self) -> bool {
        self.left_child.is_none() && self.right_child.is_none()
    }

    /// Check if this node has only a left child
    pub fn has_only_left_child(&self) -> bool {
        self.left_child.is_some() && self.right_child.is_none()
    }

    /// Check if this node has only a right child
    pub fn has_only_right_child(&self) -> bool {
        self.left_child.is_none() && self.right_child.is_some()
    }

    /// Check if this node has both children
    pub fn has_both_children(&self) -> bool {
        self.left_child.is_some() && self.right_child.is_some()
    }
}

impl Default for AvlNode {
    fn default() -> Self {
        Self::new()
    }
}

/// AVL Tree operations trait
pub trait AvlTree<T> {
    /// Get the price for comparison
    fn get_price(&self, index: usize) -> Price;
    
    /// Get the AVL node data
    fn get_node(&self, index: usize) -> &AvlNode;
    
    /// Get mutable AVL node data
    fn get_node_mut(&mut self, index: usize) -> &mut AvlNode;
    
    /// Calculate height of a subtree
    fn calculate_height(&self, index: Option<usize>) -> i32 {
        match index {
            Some(idx) => self.get_node(idx).height,
            None => 0,
        }
    }
    
    /// Update height of a node based on its children
    fn update_height(&mut self, index: usize) {
        let left_height = self.calculate_height(self.get_node(index).left_child);
        let right_height = self.calculate_height(self.get_node(index).right_child);
        self.get_node_mut(index).height = 1 + left_height.max(right_height);
    }
    
    /// Calculate balance factor (right_height - left_height)
    fn balance_factor(&self, index: usize) -> i32 {
        let node = self.get_node(index);
        let left_height = self.calculate_height(node.left_child);
        let right_height = self.calculate_height(node.right_child);
        right_height - left_height
    }
    
    /// Find minimum node in subtree
    fn find_min(&self, mut index: usize) -> usize {
        while let Some(left) = self.get_node(index).left_child {
            index = left;
        }
        index
    }
    
    /// Find maximum node in subtree
    fn find_max(&self, mut index: usize) -> usize {
        while let Some(right) = self.get_node(index).right_child {
            index = right;
        }
        index
    }
    
    /// Left rotation
    fn rotate_left(&mut self, x_index: usize) -> usize {
        let y_index = self.get_node(x_index).right_child.expect("Right child must exist for left rotation");
        
        // Store references before mutation
        let x_parent = self.get_node(x_index).parent;
        let y_left = self.get_node(y_index).left_child;
        
        // Perform rotation
        self.get_node_mut(x_index).right_child = y_left;
        self.get_node_mut(y_index).left_child = Some(x_index);
        
        // Update parents
        if let Some(y_left_idx) = y_left {
            self.get_node_mut(y_left_idx).parent = Some(x_index);
        }
        self.get_node_mut(x_index).parent = Some(y_index);
        self.get_node_mut(y_index).parent = x_parent;
        
        // Update parent's child pointer
        if let Some(parent_idx) = x_parent {
            if self.get_node(parent_idx).left_child == Some(x_index) {
                self.get_node_mut(parent_idx).left_child = Some(y_index);
            } else {
                self.get_node_mut(parent_idx).right_child = Some(y_index);
            }
        }
        
        // Update heights
        self.update_height(x_index);
        self.update_height(y_index);
        
        y_index
    }
    
    /// Right rotation
    fn rotate_right(&mut self, y_index: usize) -> usize {
        let x_index = self.get_node(y_index).left_child.expect("Left child must exist for right rotation");
        
        // Store references before mutation
        let y_parent = self.get_node(y_index).parent;
        let x_right = self.get_node(x_index).right_child;
        
        // Perform rotation
        self.get_node_mut(y_index).left_child = x_right;
        self.get_node_mut(x_index).right_child = Some(y_index);
        
        // Update parents
        if let Some(x_right_idx) = x_right {
            self.get_node_mut(x_right_idx).parent = Some(y_index);
        }
        self.get_node_mut(y_index).parent = Some(x_index);
        self.get_node_mut(x_index).parent = y_parent;
        
        // Update parent's child pointer
        if let Some(parent_idx) = y_parent {
            if self.get_node(parent_idx).left_child == Some(y_index) {
                self.get_node_mut(parent_idx).left_child = Some(x_index);
            } else {
                self.get_node_mut(parent_idx).right_child = Some(x_index);
            }
        }
        
        // Update heights
        self.update_height(y_index);
        self.update_height(x_index);
        
        x_index
    }
    
    /// Balance a node and return the new root of the subtree
    fn balance(&mut self, index: usize) -> usize {
        self.update_height(index);
        let balance = self.balance_factor(index);
        
        if balance > 1 {
            // Right heavy
            let right_child = self.get_node(index).right_child.unwrap();
            if self.balance_factor(right_child) < 0 {
                // Right-Left case
                self.rotate_right(right_child);
            }
            // Right-Right case
            self.rotate_left(index)
        } else if balance < -1 {
            // Left heavy
            let left_child = self.get_node(index).left_child.unwrap();
            if self.balance_factor(left_child) > 0 {
                // Left-Right case
                self.rotate_left(left_child);
            }
            // Left-Left case
            self.rotate_right(index)
        } else {
            // Already balanced
            index
        }
    }
}
