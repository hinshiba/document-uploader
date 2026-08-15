import {
    configureSync,
    defaultConsoleFormatter,
    getConsoleSink,
    getLogger,
    isLogLevel,
    type LogLevel,
} from "@logtape/logtape";

declare module "bun" {
    interface Env {
        /** ログレベル */
        BUN_PUBLIC_LOG_LEVEL?: string;
    }
}

const DEFAULT_LOG_LEVEL = "info" satisfies LogLevel;
const envLogLevel = process.env.BUN_PUBLIC_LOG_LEVEL;
const logLevel: LogLevel =
    envLogLevel !== undefined && isLogLevel(envLogLevel) ? envLogLevel : DEFAULT_LOG_LEVEL;

configureSync({
    reset: true,
    sinks: {
        console: getConsoleSink({
            formatter: (r) => [...defaultConsoleFormatter(r), r.properties],
        }),
    },
    loggers: [
        // logtape自身のログを分離する
        { category: ["logtape", "meta"], lowestLevel: "warning", sinks: ["console"] },

        // 継承によって自動で[app, ...]にも反映される
        { category: ["app"], lowestLevel: logLevel, sinks: ["console"] },
    ],
});

/** 各カテゴリ設定済みのLogger */
export const log = {
    api: getLogger(["app", "api"]),
    upload: getLogger(["app", "ui", "upload"]),
    download: getLogger(["app", "ui", "download"]),
    subject: getLogger(["app", "ui", "subject"]),
};
