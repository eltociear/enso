/** @file Configuration options for the application. */

import { logger } from 'runner/log'

export const DEFAULT_ENTRY_POINT = 'ide'

// =============
// === Utils ===
// =============

/** Parses the provided value as boolean. If it was a boolean value, it is left intact. If it was
 * a string 'true', 'false', '1', or '0', it is converted to a boolean value. Otherwise, null is
 * returned. */
// prettier-ignore
function parseBoolean(value: any): boolean | null {
    switch(value) {
        case true: return true
        case false: return false
        case 'true': return true
        case 'false': return false
        case 'enabled': return true
        case 'disabled': return false
        case 'yes': return true
        case 'no': return false
        case '1': return true
        case '0': return false
        default: return null
    }
}

// ==============
// === Option ===
// ==============

/** A valid parameter value. */
export type OptionValue = string | boolean | number | (string | null)

export type OptionType = 'string' | 'boolean' | 'number'

/** Configuration parameter. */
export class Option<T> {
    name = 'uninitialized'
    group = 'uninitialized'
    default: T
    value: T
    type: OptionType
    description: string
    /** The description of the default argument that should be shown to the user. For example,
     * it can be set to `'true' on macOS and 'false' otherwise` to better explain mechanics for the
     * default value. */
    defaultDescription: null | string = null
    setByUser = false
    hidden: boolean
    /** Controls whether this option should be visible by default in the help message. Non-primary
     * options will be displayed on-demand only. */
    primary = true
    constructor(cfg: {
        default: T
        type: OptionType
        description: string
        defaultDescription?: string
        hidden?: boolean
        primary?: boolean
    }) {
        this.default = cfg.default
        this.value = cfg.default
        this.description = cfg.description
        this.defaultDescription = cfg.defaultDescription ?? null
        this.type = cfg.type
        this.hidden = cfg.hidden ?? false
        this.primary = cfg.primary ?? true
    }

    qualifiedName(): string {
        return this.group && this.group != this.name ? `${this.group}.${this.name}` : this.name
    }

    load(input: string) {
        if (typeof this.value === 'boolean') {
            const newVal = parseBoolean(input)
            if (newVal == null) {
                this.printValueUpdateError(input)
            } else {
                this.value = newVal as T
                this.setByUser = true
            }
        } else if (typeof this.value == 'number') {
            const newVal = Number(input)
            if (isNaN(newVal)) {
                this.printValueUpdateError(input)
            } else {
                this.value = newVal as T
                this.setByUser = true
            }
        } else {
            this.value = String(input) as T
            this.setByUser = true
        }
    }

    printValueUpdateError(input: string) {
        logger.error(
            `The provided value for '${this.qualifiedName()}' is invalid. Expected ${this.type}, \
            got '${input}'. Using the default value '${String(this.default)}' instead.`
        )
    }
}

// ==============
// === Options ===
// ==============

export type ExternalConfig = Record<string, OptionValue>

export class OptionGroups {
    static loader = 'loader'
    static startup = 'startup'
    static debug = 'debug'
}

// export type ExternalOptions = Record<string, Record<string, Option<OptionValue>>>

interface ExternalOptions {
    [key: string]: Option<OptionValue> | ExternalOptions
}

type OptionsRecord = Record<string, Option<OptionValue>>
type GroupsRecord = Record<string, GroupLike>

interface GroupLike {
    options: OptionsRecord
    groups: GroupsRecord
    merge<Other extends GroupLike>(other: Other): this & Other
}

class Group<Options extends OptionsRecord, Groups extends GroupsRecord> {
    options: Options
    groups: Groups
    constructor(cfg?: { options?: Options; groups?: Groups }) {
        this.options = cfg?.options ?? ({} as Options)
        this.groups = cfg?.groups ?? ({} as Groups)
    }

    merge<Other extends GroupLike>(other: Other): this & Other {
        const result: GroupLike = new Group()

        Object.assign(result.groups, this.groups)
        for (const [otherGroupName, otherGroup] of Object.entries(other.groups)) {
            const group = result.groups[otherGroupName]
            if (group == null) {
                result.groups[otherGroupName] = otherGroup
            } else {
                result.groups[otherGroupName] = group.merge(otherGroup)
            }
        }
        Object.assign(result.options, this.options)
        for (const [otherOptionName, otherOption] of Object.entries(other.options)) {
            const option = result.options[otherOptionName]
            if (option != null) {
                // TODO warning
            }
            result.options[otherOptionName] = otherOption
        }
        return result as this & Other
    }

    load(config: unknown) {
        if (typeof config === 'object' && config != null) {
            for (const [key, value] of Object.entries(config)) {
                if (typeof value === 'string') {
                    const option = this.options[key]
                    if (option == null) {
                        console.error('TODO')
                    } else {
                        option.load(value)
                    }
                } else {
                    console.error('TODO')
                }
            }
        } else {
            console.error('TODO')
        }
    }
}

const opt1 = new Group({
    options: {
        opt1: new Option<boolean>({
            default: false,
            type: 'boolean',
            description: 'foo',
        }),
    },
    groups: {
        grp1: new Group({
            options: {
                grp1Opt1: new Option<boolean>({
                    default: false,
                    type: 'boolean',
                    description: 'foo',
                }),
            },
        }),
    },
})

const opt2 = new Group({
    options: {
        opt2: new Option<boolean>({
            default: false,
            type: 'boolean',
            description: 'foo',
        }),
    },
    groups: {
        grp1: new Group({
            options: {
                grp1Opt2: new Option<boolean>({
                    default: false,
                    type: 'boolean',
                    description: 'foo',
                }),
            },
        }),
    },
})

const xx = opt1.merge(opt2)
console.log('XXXXXX', xx.groups.grp1.options)
const yy = xx.groups.grp1.options.grp1Opt2.value === true

/** Application default configuration. Users of this library can extend it with their own
 * options. */
export const options = {
    loader: {
        enabled: new Option<boolean>({
            type: 'boolean',
            default: true,
            description:
                'Controls whether the visual loader should be visible on the screen when ' +
                'downloading and compiling WASM sources. By default, the loader is used only if ' +
                `the \`entry\` is set to ${DEFAULT_ENTRY_POINT}.`,
            primary: false,
        }),
        pkgWasmUrl: new Option<string>({
            type: 'string',
            default: 'pkg.wasm',
            description: 'The URL of the WASM pkg file generated by ensogl-pack.',
            primary: false,
        }),
        pkgJsUrl: new Option<string>({
            type: 'string',
            default: 'pkg.js',
            description: 'The URL of the JS pkg file generated by ensogl-pack.',
            primary: false,
        }),
        shadersUrl: new Option<string>({
            type: 'string',
            default: 'shaders',
            description: 'The URL of pre-compiled the shaders directory.',
            primary: false,
        }),
        loaderDownloadToInitRatio: new Option<number>({
            type: 'number',
            default: 1.0,
            description:
                'The (time needed for WASM download) / (total time including WASM ' +
                'download and WASM app initialization). In case of small WASM apps, this can be set ' +
                'to 1.0. In case of bigger WASM apps, it is desired to show the progress bar growing ' +
                'up to e.g. 70% and leaving the last 30% for WASM app init.',
            primary: false,
        }),
        maxBeforeMainTimeMs: new Option<number>({
            type: 'number',
            default: 300,
            description:
                'The maximum time in milliseconds a before main entry point is allowed to run. After ' +
                'this time, an error will be printed, but the execution will continue.',
            primary: false,
        }),
    },

    // === Application Startup Options ===

    startup: {
        entry: new Option<string>({
            type: 'string',
            default: DEFAULT_ENTRY_POINT,
            description:
                'The application entry point. Use `entry=_` to list available entry points.',
        }),
        theme: new Option<string>({
            type: 'string',
            default: 'default',
            description: 'The EnsoGL theme to be used.',
        }),
    },

    debug: {
        debug: new Option<boolean>({
            type: 'boolean',
            default: false,
            description:
                'Controls whether the application should be run in the debug mode. In this mode all ' +
                'logs are printed to the console. Otherwise, the logs are hidden unless explicitly ' +
                'shown by calling `showLogs`. Moreover, EnsoGL extensions are loaded in the debug ' +
                'mode which may cause additional logs to be printed.',
        }),
        enableSpector: new Option<boolean>({
            type: 'boolean',
            default: false,
            description:
                'Enables SpectorJS. This is a temporary flag to test Spector. It will be removed ' +
                'after all Spector integration issues are resolved. See: ' +
                'https://github.com/BabylonJS/Spector.js/issues/252.',
            primary: false,
        }),
    },
}

export type Options = typeof options & ExternalOptions

export function mergeOptions<T1 extends Options, T2 extends Options>(
    opts1: T1,
    opts2: T2
): T1 & T2 {
    const result: ExternalOptions = {}

    for (const [group, options] of Object.entries(opts1)) {
        result[group] = options
    }
    for (const [group, options] of Object.entries(opts2)) {
        if (result[group]) {
            result[group] = Object.assign({ ...result[group] }, options)
        } else {
            result[group] = options
        }
    }
    return result as T1 & T2
}

function initOptions(options: ExternalOptions) {
    for (const [key, value] of Object.entries(options)) {
        if (value instanceof Option) {
            value.group = 'TODO'
            value.name = key
        } else {
            initOptions(value)
        }
    }
}

// ==============
// === Config ===
// ==============

/** The configuration of the EnsoGL application. The options can be overriden by the user. The
 * implementation automatically casts the values to the correct types. For example, if an option
 * override for type boolean was provided as `'true'`, it will be parsed automatically. Moreover,
 * it is possible to extend the provided option list with custom options. See the `extend` method
 * to learn more. */
export class Config {
    options: Options

    constructor(inputOptions?: Options) {
        this.options = inputOptions || options
        initOptions(this.options)
    }

    /** Resolve the configuration from the provided record list.
     * @returns list of unrecognized parameters. */
    resolve(overrides: (Record<string, Record<string, any>> | undefined)[]): null | string[] {
        const allOverrides: Record<string, Record<string, any>> = {}
        for (const override of overrides) {
            if (override != null) {
                for (const [group, options] of Object.entries(override)) {
                    const overridesGroup = allOverrides[group] || {}
                    allOverrides[group] = Object.assign(overridesGroup, options)
                }
            }
        }
        const unrecognizedParams = this.resolveFromObject(allOverrides)
        this.finalize()
        return unrecognizedParams
    }

    /** Resolve the configuration from the provided record.
     * @returns list of unrecognized parameters. */

    resolveFromObject(other: Record<string, unknown>): null | string[] {
        return this.resolveFromObjectInternal(this.options, other)
    }

    resolveFromObjectInternal(options: ExternalOptions, other: unknown): null | string[] {
        if (typeof other === 'object' && other != null) {
            for (const [key, value] of Object.entries(other)) {
                const option = options[key]
                if (option == null) {
                    // TODO
                } else {
                    if (typeof value === 'string') {
                        if (option instanceof Option) {
                            option.value = value // FIXME parsing
                        } else {
                            // TODO
                        }
                    } else {
                        if (option instanceof Option) {
                            // TODO
                        } else {
                            this.resolveFromObjectInternal(option, value)
                            // TODO
                        }
                    }
                }
            }
        } else {
            // TODO
        }

        return null
        // const paramsToBeAssigned = new Map(
        //     Object.entries(other).map(([group, options]) => [group, new Set(Object.keys(options))])
        // )
        // for (const [group, options] of Object.entries(this.options)) {
        //     const otherGroup = other[group]
        //     const groupOfParamsToBeAssigned = paramsToBeAssigned.get(group)
        //     if (otherGroup != null && groupOfParamsToBeAssigned != null) {
        //         for (const key of Object.keys(options)) {
        //             groupOfParamsToBeAssigned.delete(key)
        //             const otherVal: unknown = otherGroup[key]
        //             const option = options[key]
        //             if (option != null && otherVal != null) {
        //                 const selfVal = option.value
        //                 if (typeof selfVal === 'boolean') {
        //                     const newVal = parseBoolean(otherVal)
        //                     if (newVal == null) {
        //                         this.printValueUpdateError(key, selfVal, otherVal)
        //                     } else {
        //                         option.value = newVal
        //                         option.setByUser = true
        //                     }
        //                 } else if (typeof selfVal == 'number') {
        //                     const newVal = Number(otherVal)
        //                     if (isNaN(newVal)) {
        //                         this.printValueUpdateError(key, selfVal, otherVal)
        //                     } else {
        //                         option.value = newVal
        //                         option.setByUser = true
        //                     }
        //                 } else {
        //                     option.value = String(otherVal)
        //                     option.setByUser = true
        //                 }
        //             }
        //         }
        //     }
        // }

        // const x = Array.from(paramsToBeAssigned.entries())
        // const unrecognized = x.flatMap(([group, options]) =>
        //     Array.from(options.values(), option =>
        //         group === option ? group : group + '.' + option
        //     )
        // )
        // if (unrecognized.length > 0) {
        //     return unrecognized
        // } else {
        //     return null
        // }
    }

    /** Finalize the configuration. Set some default options based on the provided values. */
    finalize() {
        if (
            !this.options.loader.enabled.setByUser &&
            this.options.startup.entry.value !== DEFAULT_ENTRY_POINT
        ) {
            this.options.loader.enabled.value = false
        }
    }

    printValueUpdateError(key: string, selfVal: any, otherVal: any) {
        console.error(
            `The provided value for Config.${key} is invalid. Expected boolean, got '${otherVal}'. \
            Using the default value '${selfVal}' instead.`
        )
    }

    strigifiedKeyValueMap(): Record<string, Record<string, any>> {
        // const config: Record<string, Record<string, any>> = {}
        // for (const [group, options] of Object.entries(this.options)) {
        //     const configGroup: Record<string, any> = {}
        //     config[group] = configGroup
        //     for (const [key, option] of Object.entries(options)) {
        //         if (option.value != null) {
        //             configGroup[key] = option.value.toString()
        //         } else {
        //             configGroup[key] = option.value
        //         }
        //     }
        // }
        // return config
        return {}
    }

    print() {
        logger.log(`Resolved config:`, this.strigifiedKeyValueMap())
    }
}
