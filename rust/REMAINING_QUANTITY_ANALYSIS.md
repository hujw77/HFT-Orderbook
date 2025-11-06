# remaining_quantity 字段分析

## 当前实现 vs C/Python 实现

### C 实现 (`hftlob.h`)
```c
typedef struct Order{
    double shares;  // 只有一个数量字段，直接修改
    // ...
} Order;
```

### Python 实现 (`lob.py`)
```python
class Order:
    __slots__ = ['uid', 'is_bid', 'size', 'price', ...]  # size 是唯一数量字段
    # size 直接修改，不区分原始和剩余
```

### Rust 当前实现 (`order.rs`)
```rust
pub struct Order {
    pub quantity: Quantity,           // 原始订单数量（不变）
    pub remaining_quantity: Quantity, // 剩余数量（可变）
    // ...
}
```

## 为什么 Rust 实现有两个字段？

### 1. **支持订单审计和状态跟踪**
- `quantity`: 记录用户最初提交的订单数量（不可变）
- `remaining_quantity`: 记录当前剩余数量（可变）
- 可以计算已成交数量：`filled_quantity = quantity - remaining_quantity`

### 2. **支持部分成交状态**
```rust
pub fn is_partially_filled(&self) -> bool {
    self.remaining_quantity > 0 && self.remaining_quantity < self.quantity
}
```

### 3. **订单状态管理**
```rust
if self.remaining_quantity == 0 {
    self.status = OrderStatus::Filled;
} else if self.remaining_quantity < self.quantity {
    self.status = OrderStatus::PartiallyFilled;
}
```

## 能否去掉 remaining_quantity？

### 方案 1: 简化版本（更像 C/Python）

**改动**：
- 只保留 `quantity` 字段
- 成交时直接修改 `quantity`
- 去掉 `remaining_quantity`、`status`、`filled_quantity()` 等方法

**优点**：
- ✅ 更简单，和 C/Python 实现一致
- ✅ 减少内存占用（每个订单少 8 字节）
- ✅ 代码更直观

**缺点**：
- ❌ 失去原始订单信息（无法做订单审计）
- ❌ 无法区分"部分成交"状态
- ❌ 无法计算已成交数量

**代码示例**：
```rust
// 简化后
pub struct Order {
    pub quantity: Quantity,  // 直接修改这个字段
    // ...
}

// 成交时
order.quantity -= fill_quantity;  // 直接减少
```

### 方案 2: 保留当前实现

**优点**：
- ✅ 支持订单审计（记录原始数量）
- ✅ 支持部分成交状态跟踪
- ✅ 更符合现代交易系统需求
- ✅ 支持订单状态管理

**缺点**：
- ❌ 多一个字段（内存开销）
- ❌ 代码稍微复杂一点

## 建议

### 如果目标是**完全匹配 C/Python 实现**：
- 可以去掉 `remaining_quantity`
- 只保留 `quantity`，直接修改它
- 这样可以减少约 8 字节内存（每个订单）

### 如果目标是**更好的交易系统功能**：
- 保留当前实现
- `remaining_quantity` 提供了有价值的功能
- 现代交易系统通常需要这些信息

## 如果选择简化

需要修改的地方：

1. **Order 结构**：
```rust
pub struct Order {
    pub quantity: Quantity,  // 移除 remaining_quantity
    // ...
}
```

2. **Order::new()**：
```rust
Self {
    quantity,  // 不再需要 remaining_quantity: quantity
    // ...
}
```

3. **Order::fill()**：
```rust
pub fn fill(&mut self, quantity: Quantity, event_time: Timestamp) -> Quantity {
    let fill_quantity = quantity.min(self.quantity);  // 直接使用 quantity
    self.quantity -= fill_quantity;  // 直接修改
    // ...
}
```

4. **MatchingEngine**：
```rust
// 使用 order.quantity 而不是 order.remaining_quantity
while order.quantity > 0 {
    // ...
}
```

5. **OrderBook::update_order()**：
```rust
let old_quantity = order.quantity;  // 使用 quantity
// ...
self.limits[limit_idx].as_mut().unwrap()
    .update_order_stats(old_quantity, order.quantity);  // 使用 quantity
```

6. **移除的方法**：
- `is_partially_filled()` - 需要 remaining_quantity
- `filled_quantity()` - 需要 remaining_quantity
- `remaining_value()` - 需要 remaining_quantity
- `OrderStatus::PartiallyFilled` - 可能不再需要

## 总结

- **C/Python 风格**：只有 `quantity`，直接修改
- **当前 Rust 风格**：`quantity`（原始）+ `remaining_quantity`（剩余）

选择哪个取决于你的需求：
- 如果只是实现基本的订单簿数据结构，可以简化
- 如果需要支持订单审计、状态跟踪，保留当前实现

