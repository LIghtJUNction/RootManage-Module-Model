package build

import "errors"

// 特殊错误类型
var ErrSkipped = errors.New("跳过编译")
