use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashMap},
};

/// Узел дерева Хаффмана.
///
/// Лист содержит `symbol` и `weight`, внутренний узел содержит `weight`
/// (сумму весов потомков) и ссылки на левого/правого потомка.
pub struct Node {
    pub weight: u64,
    pub symbol: Option<u8>,
    pub left: Option<Box<Node>>,
    pub right: Option<Box<Node>>,
}

impl Eq for Node {}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.weight == other.weight
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other.weight.cmp(&self.weight)
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Node {
    /// Создаёт листовой узел с заданными символом и весом.
    pub fn new_leaf(symbol: u8, weight: u64) -> Self {
        Node {
            weight,
            symbol: Some(symbol),
            left: None,
            right: None,
        }
    }

    /// Создаёт внутренний узел с весом, равным сумме весов потомков.
    pub fn new_internal(left: Box<Node>, right: Box<Node>) -> Self {
        Node {
            weight: left.weight + right.weight,
            symbol: None,
            left: Some(left),
            right: Some(right),
        }
    }
}

/// Строит дерево Хаффмана по массиву частот (256 байт).
///
/// Использует бинарную кучу (`BinaryHeap`). Для пустого входа
/// возвращает `None`, для одного символа — дерево с корнем,
/// имеющим одного потомка-лист.
pub fn build_huffman_tree(frequencies: [u64; 256]) -> Option<Box<Node>> {
    let mut heap = BinaryHeap::new();

    for (byte, &freq) in frequencies.iter().enumerate() {
        if freq > 0 {
            heap.push(Box::new(Node::new_leaf(byte as u8, freq)));
        }
    }

    match heap.len() {
        0 => return None,
        1 => {
            let single = heap.pop().unwrap();
            return Some(Box::new(Node {
                weight: single.weight,
                symbol: None,
                left: Some(single),
                right: None,
            }));
        }
        _ => {}
    }

    while heap.len() > 1 {
        let left = heap.pop().unwrap();
        let right = heap.pop().unwrap();

        heap.push(Box::new(Node::new_internal(left, right)));
    }

    Some(heap.pop().unwrap())
}

pub type Code = Vec<bool>;
pub type CodeTable = HashMap<u8, Vec<bool>>;

pub type CodeLengths = [u8; 256];

/// Строит таблицу кодов рекурсивным обходом дерева.
///
/// Каждому листу сопоставляется путь от корня (false = влево, true = вправо).
pub fn code_table_generation(root: &Node) -> CodeTable {
    let mut table = CodeTable::new();
    generate_codes_recursive(&root, &mut Vec::new(), &mut table);
    return table;
}

/// Рекурсивная часть `code_table_generation`.
fn generate_codes_recursive(node: &Node, current_code: &mut Code, table: &mut CodeTable) {
    if let Some(symbol) = node.symbol {
        table.insert(symbol, current_code.clone());
        return;
    }

    if let Some(ref left) = node.left {
        current_code.push(false);
        generate_codes_recursive(left, current_code, table);
        current_code.pop();
    }

    if let Some(ref right) = node.right {
        current_code.push(true);
        generate_codes_recursive(right, current_code, table);
        current_code.pop();
    }
}

/// Вычисляет длины кодов Хаффмана для каждого символа (0 для отсутствующих).
pub fn get_code_lengths(root: &Node) -> CodeLengths {
    let mut lengths = [0u8; 256];
    calculate_lengths_recursive(root, 0, &mut lengths);
    lengths
}

/// Рекурсивная часть `get_code_lengths`.
fn calculate_lengths_recursive(node: &Node, current_depth: u8, lengths: &mut CodeLengths) {
    if let Some(symbol) = node.symbol {
        lengths[symbol as usize] = current_depth;
        return;
    }

    if let Some(ref left) = node.left {
        calculate_lengths_recursive(left, current_depth + 1, lengths);
    }

    if let Some(ref right) = node.right {
        calculate_lengths_recursive(right, current_depth + 1, lengths);
    }
}

/// Строит канонические коды Хаффмана по длинам кодов.
///
/// Символы сортируются по длине, затем по значению. Коды назначаются
/// последовательно: для каждой длины код сдвигается и инкрементируется.
/// Возвращает `None` при нарушении неравенства Крафта или длине ≥ 128.
pub fn build_canonical_codes(lengths: &CodeLengths) -> Option<CodeTable> {
    let mut symbols: Vec<(u8, u8)> = lengths.iter()
        .enumerate().filter(|&(_, &len)| len > 0)
        .map(|(idx, &len)| (idx as u8, len)).collect();

    symbols.sort_by(|a, b| a.1.cmp(&b.1).then(a.0.cmp(&b.0)));

    let mut table = HashMap::new();
    let mut code: u128 = 0;
    let mut prev_len = 0;

    for (symbol, len) in symbols {
        if len > 127 {
            return None;
        }
        code <<= len - prev_len;
        if code >= (1u128 << len) {
            return None;
        }
        let bits: Vec<bool> = (0..len).rev()
            .map(|i| (code >> i) & 1 != 0)
            .collect();
        table.insert(symbol, bits);
        code += 1;
        prev_len = len;
    }

    Some(table)
}

/// Восстанавливает дерево Хаффмана из канонических длин кодов.
///
/// Сначала строит канонические коды через `build_canonical_codes`,
/// затем обходит биты каждого кода, создавая узлы по мере необходимости.
/// Возвращает `None`, если коды пусты или невалидны.
pub fn build_tree_from_lengths(lengths: &CodeLengths) -> Option<Box<Node>> {
    let codes = build_canonical_codes(lengths)?;
    if codes.is_empty() {
        return None;
    }
    let mut root = Box::new(Node {
        weight: 0,
        symbol: None,
        left: None,
        right: None,
    });
    for (&symbol, bits) in &codes {
        let mut current = &mut root;
        for &bit in bits {
            let child = if !bit { &mut current.left } else { &mut current.right };
            if child.is_none() {
                *child = Some(Box::new(Node {
                    weight: 0,
                    symbol: None,
                    left: None,
                    right: None,
                }));
            }
            current = child.as_mut().unwrap();
        }
        current.symbol = Some(symbol);
    }
    Some(root)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_frequencies() {
        let freqs = [0u64; 256];
        assert!(build_huffman_tree(freqs).is_none());
    }

    #[test]
    fn single_symbol() {
        let mut freqs = [0u64; 256];
        freqs[0x41] = 10;
        let tree = build_huffman_tree(freqs).unwrap();
        let lengths = get_code_lengths(&tree);
        assert_eq!(lengths[0x41], 1);
    }

    #[test]
    fn two_symbols() {
        let mut freqs = [0u64; 256];
        freqs[0x41] = 5;
        freqs[0x42] = 5;
        let tree = build_huffman_tree(freqs).unwrap();
        let lengths = get_code_lengths(&tree);
        assert!(lengths[0x41] <= 2);
        assert!(lengths[0x42] <= 2);
        assert_eq!(lengths[0x41], lengths[0x42]);
    }

    #[test]
    fn code_table_lengths_match() {
        let mut freqs = [0u64; 256];
        freqs[0x41] = 10;
        freqs[0x42] = 5;
        freqs[0x43] = 3;
        let tree = build_huffman_tree(freqs).unwrap();
        let lengths = get_code_lengths(&tree);
        let codes = code_table_generation(&tree);
        for (&symbol, code) in &codes {
            assert_eq!(lengths[symbol as usize] as usize, code.len());
        }
    }

    #[test]
    fn canonical_roundtrip() {
        let mut freqs = [0u64; 256];
        freqs[b'a' as usize] = 10;
        freqs[b'b' as usize] = 5;
        freqs[b'c' as usize] = 3;
        freqs[b'd' as usize] = 2;
        freqs[b'e' as usize] = 1;

        let tree = build_huffman_tree(freqs).unwrap();
        let lengths = get_code_lengths(&tree);
        let canonical = build_canonical_codes(&lengths).unwrap();

        let reconstructed = build_tree_from_lengths(&lengths).unwrap();
        let reconstructed_lengths = get_code_lengths(&reconstructed);

        assert_eq!(lengths, reconstructed_lengths);
        assert_eq!(canonical.len(), 5);
    }

    #[test]
    fn canonical_property() {
        let mut freqs = [0u64; 256];
        let text = b"this is a test string for huffman coding";
        for &b in text {
            freqs[b as usize] += 1;
        }
        let tree = build_huffman_tree(freqs).unwrap();
        let lengths = get_code_lengths(&tree);
        let canonical = build_canonical_codes(&lengths).unwrap();

        let mut prev: Option<(u8, &Vec<bool>)> = None;
        let mut sorted: Vec<_> = canonical.iter().collect();
        sorted.sort_by(|a, b| a.0.cmp(b.0));

        for &(sym, code) in &sorted {
            if let Some((_, prev_code)) = prev {
                if code.len() == prev_code.len() {
                    let prev_val = prev_code.iter().fold(0u64, |acc, &b| (acc << 1) | b as u64);
                    let cur_val = code.iter().fold(0u64, |acc, &b| (acc << 1) | b as u64);
                    assert!(cur_val > prev_val, "codes must be sequential");
                }
            }
            prev = Some((*sym, code));
        }
    }

    #[test]
    fn empty_codes() {
        let lengths = [0u8; 256];
        let table = build_canonical_codes(&lengths).unwrap();
        assert!(table.is_empty());
    }

    #[test]
    fn prefix_property() {
        let mut freqs = [0u64; 256];
        let text = b"test data with various symbols";
        for &b in text {
            freqs[b as usize] += 1;
        }
        let tree = build_huffman_tree(freqs).unwrap();
        let codes = code_table_generation(&tree);

        for (_, code1) in &codes {
            for (_, code2) in &codes {
                if code1 == code2 { continue; }
                let min_len = code1.len().min(code2.len());
                if &code1[..min_len] == &code2[..min_len] {
                    assert_eq!(code1.len(), code2.len(), "prefix violation");
                }
            }
        }
    }

    #[test]
    fn build_tree_invalid_lengths_kraft() {
        let mut lengths = [0u8; 256];
        lengths[0] = 1;
        lengths[1] = 1;
        lengths[2] = 1;
        let tree = build_tree_from_lengths(&lengths);
        assert!(tree.is_none(), "build_tree_from_lengths should reject Kraft violation");
        let codes = build_canonical_codes(&lengths);
        assert!(codes.is_none());
    }

    #[test]
    fn build_tree_from_empty_lengths() {
        let lengths = [0u8; 256];
        assert!(build_tree_from_lengths(&lengths).is_none());
    }

    #[test]
    fn build_tree_single_symbol() {
        let mut lengths = [0u8; 256];
        lengths[0x42] = 1;
        let tree = build_tree_from_lengths(&lengths).unwrap();
        let result_lengths = get_code_lengths(&tree);
        assert_eq!(result_lengths[0x42], 1);
    }

    #[test]
    fn build_tree_all_lengths_present() {
        let mut lengths = [0u8; 256];
        for i in 0..128 {
            lengths[i] = 8;
        }
        let tree = build_tree_from_lengths(&lengths).unwrap();
        assert!(tree.left.is_some() || tree.right.is_some());
    }

    #[test]
    fn build_canonical_rejects_len_128() {
        let mut lengths = [0u8; 256];
        lengths[0] = 128;
        assert!(build_canonical_codes(&lengths).is_none());
    }

    #[test]
    fn build_canonical_len_127_works() {
        let mut lengths = [0u8; 256];
        lengths[0] = 1;
        lengths[1] = 127;
        let codes = build_canonical_codes(&lengths).unwrap();
        assert_eq!(codes.len(), 2);
        assert_eq!(codes[&0].len(), 1);
        assert_eq!(codes[&1].len(), 127);
    }
}