# RMM 项目 Makefile

.PHONY: help build develop clean test install

# 默认目标
help:
	@echo "RMM 项目构建命令:"
	@echo "  build      - 构建完整项目（Rust + Python）"
	@echo "  develop    - 开发模式构建"
	@echo "  clean      - 清理构建文件"
	@echo "  test       - 运行测试"
	@echo "  install    - 安装项目"
	@echo "  rust-only  - 只构建 Rust 扩展"

# 构建完整项目
build:
	python build.py build

# 开发模式构建
develop:
	python build.py develop

# 只构建 Rust 扩展
rust-only:
	python build.py build --rust-only

# 清理构建文件
clean:
	python build.py clean

# 运行测试
test: develop
	python -m pytest tests/ -v

# 安装项目
install: build
	pip install dist/*.whl

# 本地开发安装
install-dev: develop
	pip install -e .

# 检查代码格式
format:
	black src/
	isort src/

# 类型检查
typecheck:
	mypy src/pyrmm/

# 完整的CI检查
ci: clean build test
