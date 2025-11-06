# Rust 实现与 C/Python 实现的差异说明

## Order 数据结构的主要差异

### C 实现 (`hftlob.h`)
```c
typedef struct Order{
    char *tid;
    unsigned buyOrSell;
    double shares;
    double limit;
    double entryTime;
    double eventTime;
    struct Order *nextOrder;      // 原始指针
    struct Order *prevOrder;      // 原始指针
    struct Limit *parentLimit;    // 原始指针
    int exchangeId;
} Order;
```

### Python 实现 (`lob.py`)
```python
class Order:
    __slots__ = ['uid', 'is_bid', 'size', 'price', 'timestamp',
                 'next_item', 'previous_item', 'root']
    # next_item, previous_item 是对象引用
    # root 是 OrderList 对象，通过 root.parent_limit 访问 Limit
```

### Rust 实现 (`order.rs`)
```rust
pub struct Order {
    pub id: OrderId,
    pub side: Side,
    pub quantity: Quantity,
    pub remaining_quantity: Quantity,
    pub price: Price,
    pub entry_time: Timestamp,
    pub event_time: Timestamp,
    pub exchange_id: ExchangeId,
    pub status: OrderStatus,
    pub(crate) next_order_index: Option<usize>,      // 索引！
    pub(crate) prev_order_index: Option<usize>,      // 索引！
    pub(crate) parent_limit_index: Option<usize>,   // 索引！
}
```

## 为什么 Rust 使用索引而不是指针？

### 1. **Rust 的所有权系统限制**

Rust 的所有权系统不允许多个对象同时拥有同一个对象的可变引用。在 C 中：
- `Order *nextOrder` 和 `Order *prevOrder` 是原始指针，可以自由指向任何 Order
- 没有所有权概念，完全由程序员管理内存

在 Rust 中：
- 如果使用 `&mut Order` 或 `Box<Order>`，每个 Order 只能有一个所有者
- 无法实现双向链表的循环引用结构（双向链表需要多个对象引用同一个节点）

### 2. **生命周期复杂性**

如果使用 Rust 的引用（`&Order`），需要明确的生命周期标记：
```rust
// 这样写会非常复杂且容易出错
struct Order<'a> {
    next_order: Option<&'a Order<'a>>,
    prev_order: Option<&'a Order<'a>>,
    parent_limit: Option<&'a Limit<'a>>,
    // ... 生命周期标记会让代码变得非常复杂
}
```

### 3. **索引方案（Slot Map / Arena Allocator 模式）**

Rust 实现采用了 **Slot Map** 或 **Arena Allocator** 模式：

```rust
pub struct OrderBook {
    /// 所有订单存储在一个向量中
    orders: Vec<Option<Order>>,
    /// 所有 limit 存储在一个向量中
    limits: Vec<Option<Limit>>,
    // ...
}
```

**优势：**
- ✅ **内存安全**：所有数据存储在 OrderBook 中，避免了悬空指针
- ✅ **性能**：索引访问是 O(1)，和指针访问一样快
- ✅ **简单性**：不需要生命周期标记
- ✅ **缓存友好**：数据存储在连续内存中，访问模式更优化
- ✅ **易于序列化**：索引可以轻松转换为稳定的 ID

### 4. **性能对比**

| 操作 | C/Python (指针) | Rust (索引) |
|------|----------------|-----------|
| 访问下一个订单 | O(1) 指针解引用 | O(1) 向量索引 |
| 内存局部性 | 可能分散 | 连续存储，更好 |
| 安全性 | 需要手动管理 | 编译器保证 |

### 5. **实际代码示例**

**C 实现访问：**
```c
Order *next = current_order->nextOrder;
if (next != NULL) {
    // 直接访问
    next->shares = 100;
}
```

**Rust 实现访问：**
```rust
// 在 OrderBook 的方法中
if let Some(next_idx) = self.orders[order_idx].next_order_index {
    if let Some(next_order) = &mut self.orders[next_idx] {
        // 访问
        next_order.quantity = 100;
    }
}
```

虽然 Rust 代码看起来更复杂，但这是安全的，因为：
- 所有访问都通过 `OrderBook` 统一管理
- 编译器保证不会出现悬空指针
- 运行时不会出现内存错误

### 6. **额外的改进**

Rust 实现还添加了一些 C/Python 没有的字段：

```rust
pub remaining_quantity: Quantity,  // 支持部分成交
pub status: OrderStatus,          // 订单状态跟踪
```

这些字段使得 Rust 实现更符合现代交易系统的需求。

## 总结

Rust 使用索引而不是指针的主要原因是：

1. **内存安全**：Rust 的所有权系统不允许指针的循环引用
2. **性能相当**：索引访问和指针访问性能相同（都是 O(1)）
3. **更好的缓存**：连续存储提高缓存命中率
4. **易于维护**：不需要手动管理内存和生命周期

这是一种常见的 Rust 模式，在很多高性能 Rust 库中都有使用（如 `slotmap`、`generational-arena` 等）。

