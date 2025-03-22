# OpenRouter API 代理

一个使用 Rust 编写的轻量级 OpenRouter API 代理服务器，支持基于模型名称匹配自动指定 AI 提供商，并使用与 OpenAI API 兼容的格式处理请求。

## 功能特点

- **自动提供商匹配**：根据配置的模型模式自动设置适当的提供商
- **OpenAI 兼容身份验证**：使用标准的 Bearer token 认证格式
- **流式响应支持**：完全支持 SSE（Server-Sent Events）流式输出
- **灵活部署选项**：
  - HTTP 或 HTTPS 服务器
  - 可配置端口和基础 URL
- **错误处理**：全面的错误处理和友好的错误响应
- **模块化架构**：易于维护和扩展的项目结构

## 安装和运行

### 先决条件

- [Rust](https://www.rust-lang.org/tools/install) 和 Cargo (版本 1.60.0 或更高)

### 从源码构建

```bash
# 克隆仓库
git clone https://github.com/yourusername/openrouter-proxy.git
cd openrouter-proxy

# 构建项目
cargo build --release

# 运行服务器
./target/release/openrouter-proxy
```

### 使用 Cargo 运行

```bash
# HTTP 模式 (默认端口 3000)
cargo run --release

# 指定端口
cargo run --release -- --port 8080

# 指定模型到提供商的映射
cargo run --release -- --model-provider-mapping "*anthropic/claude*=Anthropic,Google" --model-provider-mapping "gpt-4=OpenAI"

# HTTPS 模式 (需要 SSL 证书)
cargo run --release -- --https --cert-path=./cert.pem --key-path=./key.pem
```

## 使用方法

### 标准 OpenAI 格式

发送与 OpenAI API 兼容的请求，如果模型名称匹配配置的模式，服务器会自动添加适当的提供商:

```bash
curl -X POST http://localhost:3000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_OPENROUTER_API_KEY" \
  -d '{
    "model": "anthropic/claude-3-opus-20240229",
    "messages": [{"role": "user", "content": "Hello!"}],
    "temperature": 0.7
  }'
```

对于匹配 "*anthropic/claude*" 模式的模型，上面的请求会被自动转换为:

```json
{
  "model": "anthropic/claude-3-opus-20240229",
  "messages": [{"role": "user", "content": "Hello!"}],
  "temperature": 0.7,
  "provider": {
    "order": ["Anthropic", "Google"]
  }
}
```

### 显式指定提供商

如果需要手动控制提供商，仍然可以在请求体中直接指定:

```bash
curl -X POST http://localhost:3000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_OPENROUTER_API_KEY" \
  -d '{
    "model": "mistralai/mixtral-8x7b-instruct",
    "messages": [{"role": "user", "content": "Hello!"}],
    "provider": {
      "order": ["OpenAI"]
    }
  }'
```

### 流式输出

支持标准的 OpenAI 流式输出格式:

```bash
curl -X POST http://localhost:3000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_OPENROUTER_API_KEY" \
  -d '{
    "model": "anthropic/claude-3-opus-20240229",
    "messages": [{"role": "user", "content": "Hello!"}],
    "stream": true
  }'
```

## API 参考

### 支持的端点

- `/v1/chat/completions`
- `/v1/embeddings`
- `/v1/models`

### 请求头

| 名称 | 描述 | 示例 |
|---|---|---|
| `Authorization` | 包含 OpenRouter API 密钥的认证头 | `Bearer sk-or-v1-...` |
| `Content-Type` | 请求内容类型 | `application/json` |

### 命令行参数

| 参数 | 说明 | 默认值 |
|---|---|---|
| `--port PORT` | 监听端口 | `3000` |
| `--https` | 启用 HTTPS | `false` |
| `--cert-path PATH` | SSL 证书路径 | 无 |
| `--key-path PATH` | SSL 私钥路径 | 无 |
| `--openrouter-base-url URL` | OpenRouter API 基础 URL | `https://openrouter.ai/api/v1` |
| `--model-provider-mapping PATTERN=PROVIDER1,PROVIDER2` | 模型模式到提供商的映射 | 无 |

模型模式支持以下通配符匹配:
- `*suffix` - 匹配以 "suffix" 结尾的模型名称
- `prefix*` - 匹配以 "prefix" 开头的模型名称
- `*substring*` - 匹配包含 "substring" 的模型名称
- 精确匹配 - 没有通配符时进行精确匹配

## 项目结构

```
openrouter-proxy/
├── Cargo.toml
├── Dockerfile
├── .env.example
├── README.md
├── src/
│   ├── main.rs                 # 应用入口点
│   ├── config.rs               # 配置处理
│   ├── app.rs                  # 应用设置和服务器初始化
│   ├── error.rs                # 错误类型和处理
│   ├── handlers/               # 请求处理器
│   │   ├── mod.rs              # 模块声明
│   │   ├── health.rs           # 健康检查处理器
│   │   └── proxy.rs            # 代理处理器
│   ├── models/                 # 数据模型
│   │   ├── mod.rs              # 模块声明
│   │   └── request.rs          # 请求模型
│   ├── services/               # 业务逻辑
│   │   ├── mod.rs              # 模块声明
│   │   └── openrouter.rs       # OpenRouter服务逻辑
│   └── utils/                  # 工具函数
│       ├── mod.rs              # 模块声明
│       └── tls.rs              # TLS相关工具函数
```

## 许可证

MIT License - 详情请参阅 LICENSE 文件。

## 致谢

- [OpenRouter API](https://openrouter.ai/) - 提供统一的 AI 模型访问
- [Axum](https://github.com/tokio-rs/axum) - Rust Web 框架
- [Tokio](https://tokio.rs/) - Rust 异步运行时

---

如果您有任何问题或建议，欢迎创建 issue 或提交 PR！