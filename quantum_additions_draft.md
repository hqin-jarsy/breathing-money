# Breathing Money v0.9 — Quantum Section Additions
# For Han's review before inserting into both whitepapers

## ============================================
## ENGLISH — Insert after the DAA paragraph in "Second Chisel: Extraction Function"
## (after "alternative DAA proposals are welcome, especially under shared-hash assumptions.")
## ============================================

### Signature Scheme and Post-Quantum Migration

This prototype uses secp256k1 ECDSA, the same elliptic curve signature scheme as Bitcoin and Ethereum. **This is a placeholder, not a constitutional commitment.**

On March 30, 2026, Google Quantum AI published resource estimates for breaking the 256-bit Elliptic Curve Discrete Logarithm Problem on secp256k1 (Babbush et al., "Securing Elliptic Curve Cryptocurrencies against Quantum Vulnerabilities"). Their result: Shor's algorithm can execute with ~1,200 logical qubits and ~90 million Toffoli gates, or ~1,450 logical qubits and ~70 million Toffoli gates. On superconducting architectures with current error rates, this translates to fewer than 500,000 physical qubits — roughly a 20x reduction from prior estimates. The paper introduces the concept of "on-spend" attacks: a quantum adversary who can derive a private key from a public key faster than a transaction confirms.

Bitcoin is already responding. BIP-360 (Pay-to-Merkle-Root) has been merged into Bitcoin's official BIP repository, removing Taproot's quantum-vulnerable keypath spend. BTQ Technologies has deployed a working implementation on testnet with Dilithium post-quantum signature opcodes. But BIP-360 only addresses long-range attacks (exposed static public keys), not on-spend attacks. And BIP-360 co-author Ethan Heilman estimates even an optimistic full migration would take Bitcoin approximately seven years — SegWit took 8.5 years from conception to adoption, Taproot 7.5.

**This chain does not yet exist. That is an advantage.** We can build the migration path into genesis rather than retrofit it onto a live network with billions of dollars at stake.

What this means for this design:

1. **The signature scheme belongs to the maintainable layer, not the monetary constitution.** Block space policy, supply function, extraction function, and floor constraints are immutable. The signing algorithm is engineering — it can and should be replaced when the threat materializes.

2. **Address format must support migration from genesis.** The protocol should define addresses as algorithm-agnostic: a version byte that selects the signature verification rule. This allows a soft-fork upgrade path to post-quantum signatures (NIST candidates: ML-DSA/Dilithium, FN-DSA/FALCON, SLH-DSA/SPHINCS+) without changing the monetary layer. Bitcoin needs BIP-360 to retrofit this capability; we can have it from block 0.

3. **On-spend attacks interact with block time.** A 10-minute target block time provides a larger window for quantum key extraction than faster chains. If post-quantum signatures are not adopted before CRQCs arrive, the chain would need to transition to hash-based address commitments (spend-to-reveal with time lock) as an interim defense. This is a known engineering problem with known solutions.

4. **The supply signal is partially affected.** If on-spend attacks enable unauthorized transfers, the resulting on-chain volume is economically meaningless noise. However, the 200-month MA provides substantial dampening, and the attack requires per-transaction quantum computation — mass volume fabrication via quantum attacks is not the economical path (direct theft is). The supply signal's vulnerability to quantum attacks is second-order compared to the signature scheme itself.

**We do not claim quantum resistance. We claim the design separates the parts that must be immutable (monetary policy) from the parts that must be upgradeable (cryptographic primitives), and that this separation is deliberate.**

**Help wanted:** What is the right address format to support algorithm-agile signatures from genesis? What are the concrete trade-offs between ML-DSA, FN-DSA, and SLH-DSA for a UTXO-based PoW chain with 10-minute blocks? Post-quantum signatures are significantly larger (Dilithium: ~2.4 KB, SPHINCS+: ~7-40 KB vs ECDSA: ~72 bytes) — what are the block space implications?


## ============================================
## ENGLISH — Add to "Where I Need Help" list, as new item 10 (shift existing 10 to 11)
## ============================================

10. **Post-quantum signature migration path** — Algorithm-agile address format from genesis. Which PQC signature scheme best fits a UTXO PoW chain? What are the block weight implications of ~2.4 KB (Dilithium) or ~7+ KB (SPHINCS+) signatures?


## ============================================
## ENGLISH — Add to "What This Is Not" paragraph, at the end
## ============================================

Not quantum-resistant (secp256k1 is a placeholder; the design separates monetary constitution from cryptographic primitives to enable migration).


## ============================================
## CHINESE — Insert after DAA paragraph in "第二凿: 挖掘函数"
## (after "欢迎提出替代的DAA和算法方案。")
## ============================================

### 签名方案与后量子迁移

当前原型使用secp256k1 ECDSA，和比特币、以太坊相同的椭圆曲线签名方案。**这是占位符，不是宪法承诺。**

2026年3月30日，Google Quantum AI发布了破解secp256k1上256位椭圆曲线离散对数问题的资源估算（Babbush等，"Securing Elliptic Curve Cryptocurrencies against Quantum Vulnerabilities"）。结论: Shor算法可以在约1,200个逻辑量子比特配合约9,000万Toffoli门，或约1,450个逻辑量子比特配合约7,000万Toffoli门的条件下执行。在当前错误率的超导架构上，这意味着不到50万个物理量子比特，比此前估计降低了约20倍。论文引入了"on-spend"攻击的概念: 量子攻击者在交易确认之前就从公钥推导出私钥。

比特币已经在行动。BIP-360（Pay-to-Merkle-Root）已经合并进比特币官方BIP仓库，移除了Taproot中量子脆弱的keypath spend。BTQ Technologies已在测试网上部署了包含Dilithium后量子签名操作码的可用实现。但BIP-360只解决long-range攻击（暴露的静态公钥），不解决on-spend攻击。而且BIP-360联合作者Ethan Heilman估计，即使乐观情况下，比特币完成完整迁移也需要大约七年——SegWit从提出到采用花了8.5年，Taproot花了7.5年。

**这条链还不存在，这是一个优势。** 我们可以把迁移路径从创世就建进去，而不是在一个承载数千亿美元的活跃网络上做改造。

这对本设计的意义:

1. **签名方案属于可维护层，不属于货币宪法。** 区块空间策略、供给函数、挖掘函数和下限约束是不可变的。签名算法是工程层面的，它可以也应当在威胁到来时被替换。

2. **地址格式必须从创世就支持迁移。** 协议应定义算法无关的地址格式: 一个版本字节选择签名验证规则。这允许通过软分叉升级到后量子签名（NIST候选方案: ML-DSA/Dilithium, FN-DSA/FALCON, SLH-DSA/SPHINCS+），无需改变货币层。比特币需要BIP-360来补救这个能力，我们可以从第0个区块就拥有它。

3. **On-spend攻击与出块时间相关。** 10分钟的目标出块时间给量子密钥提取提供了比快速链更大的窗口。如果在CRQC到来之前未采用后量子签名，链需要过渡到基于哈希的地址承诺方案（先承诺后揭示，带时间锁）作为临时防御。这是一个已知的工程问题，有已知的解决方案。

4. **供给信号受到部分影响。** 如果on-spend攻击使未授权转账成为可能，由此产生的链上交易量是无经济意义的噪声。但200个月MA提供了充分的阻尼，而且攻击需要逐笔交易进行量子计算——通过量子攻击大规模制造虚假交易量不是经济上合理的路径（直接盗取才是）。供给信号对量子攻击的脆弱性，相比签名方案本身是二阶问题。

**我们不宣称量子抗性。我们宣称的是: 设计将必须不可变的部分（货币政策）和必须可升级的部分（密码学原语）分离开来，这种分离是有意为之的。**

**求助:** 什么样的地址格式能从创世开始支持算法灵活的签名？ML-DSA、FN-DSA和SLH-DSA对一个基于UTXO的PoW链（10分钟出块）各有什么具体权衡？后量子签名体积显著更大（Dilithium约2.4 KB, SPHINCS+约7-40 KB，而ECDSA约72字节），对区块空间有什么影响？


## ============================================
## CHINESE — Add to "我需要帮助的地方" list, as new item 11
## ============================================

11. **后量子签名迁移路径** — 从创世开始的算法灵活地址格式。哪种PQC签名方案最适合UTXO PoW链？约2.4 KB（Dilithium）或7+ KB（SPHINCS+）的签名对区块重量有什么影响？


## ============================================
## CHINESE — Add to "这不是什么" paragraph, at the end
## ============================================

不是量子安全的（secp256k1是占位符; 设计将货币宪法和密码学原语分离，以支持迁移）。
