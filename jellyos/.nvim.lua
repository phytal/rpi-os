local nvim_lsp = require('lspconfig')

nvim_lsp.rust_analyzer.setup {
    settings = {
        ["rust-analyzer"] = {
            cargo = {
                target = "aarch64-unknown-none",
                features = "all",
            },
            checkOnSave = {
                allTargets = false,
            },
        },
    },
}

