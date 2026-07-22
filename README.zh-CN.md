<p align="center">
  <a href="README.md">English</a> · <a href="README.zh-CN.md">中文</a> · <a href="README.ru.md">Русский</a> · <a href="README.ja.md">日本語</a>
</p>

# ProofCat

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="docs/assets/hero-dark.png">
    <img alt="ProofCat — 先证明拷贝可靠，再重新使用存储卡" src="docs/assets/hero-light.png">
  </picture>
</p>

<p align="center"><strong>在重新使用存储卡之前，先完成两份经过验证的副本。</strong></p>

<p align="center">
  面向 macOS 和 Windows 的免费离线拍摄卡备份工具。<br>
  <a href="https://github.com/reynikman/proofcat/releases/tag/v0.3.0"><strong>下载 ProofCat 0.3.0</strong></a>
  · <a href="docs/TECHNICAL.md">技术文档（英文）</a>
</p>

拍摄结束后，ProofCat 会把存储卡复制到你选择的硬盘，独立检查这些副本，
并给出清晰结论。它不会替你格式化任何设备；它只会告诉你证据是否足以重新使用这张卡。

## 一个明确的结论

1. 选择拍摄卡和两块目标硬盘。
2. ProofCat 复制并检查每一个必需文件。
3. 只有当应用显示 **SAFE TO FORMAT** 时，才重新使用存储卡。

只要有文件缺失、检查失败、任务中断、磁盘已满，或配置存在歧义，
该结论就不会出现。同一块物理硬盘上的两个文件夹不算两份备份。

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="docs/assets/verdict-dark.png">
    <img alt="ProofCat 仅在所有必要检查通过后显示 SAFE TO FORMAT" src="docs/assets/verdict-light.png">
  </picture>
</p>

## 为拍摄结束后的关键时刻而设计

- **默认离线。** 媒体文件留在本机。
- **两块真实目标设备。** 检查的是设备，而不只是文件夹名称。
- **继续，而不是猜测。** 重新连接硬盘后可继续中断的任务。
- **可交付的证据。** 为每次复制保留可读报告。
- **不止备份。** 同一应用还能检查媒体并导出元数据与交付报告。

## 获取 ProofCat

**ProofCat 0.3.0** 支持 **Apple Silicon Mac** 和 **Windows x64**。请在
[发布页面](https://github.com/reynikman/proofcat/releases/tag/v0.3.0)下载对应安装程序。

首次启动时，macOS 可能显示 Gatekeeper 提示，Windows 可能显示 SmartScreen 提示；
当前版本尚未完成 Apple notarization 或 Windows Authenticode 签名。发布页面提供校验和与签名，
[技术文档](docs/TECHNICAL.md#installation-and-release-integrity)说明如何验证。

## 需要技术细节？

产品把简单承诺与工程证据分开呈现。以下技术资料均为英文：

| 问题 | 阅读 |
|---|---|
| `SAFE TO FORMAT` 的确切含义是什么？ | [安全契约](docs/offload-guarantees.md) |
| 复制和验证流程如何工作？ | [技术文档](docs/TECHNICAL.md) |
| 真机测试结果如何？ | [硬件测试报告](docs/TEST_REPORT.md) |
| 系统的边界和限制是什么？ | [威胁模型](docs/threat-model.md) |
| 与成熟工具相比如何？ | [诚实比较](docs/COMPARISON.md) |
| 如何构建或贡献？ | [贡献指南](CONTRIBUTING.md) |

## 开源，并对结论负责

ProofCat 使用 [MIT 许可证](LICENSE)。一个决定拍摄卡能否重新使用的工具，
其源代码应当可以被检查。发现问题请提交 [GitHub issue](https://github.com/reynikman/proofcat/issues)；
安全问题请使用 [私密漏洞报告](SECURITY.md)。
