export { compile, isValidIdentifier, start, prepareStackTrace } from './pkg/compiler';

declare interface JsMethodParameter {
    name?: String;
    index: number;
    hasDefault: boolean;
    isObjectPattern: boolean;
    isArrayPattern: boolean;
    isRestElement: boolean;
}

declare interface JsMemberData {
    memberIndex: number;
    kind: 'method' | 'field' | 'accessor' | 'getter' | 'setter';
    name: string | symbol;
    static?: boolean;
    private?: boolean;
    access?: { get?: () => any; set?: (v: any) => void };
    parameters?: JsMethodParameter[];
    docblock?: String;
}

declare interface JsReflectionData {
    fqcn: String;
    className: String;
    namespace?: String;
    filename?: String;
    members: JsMemberData[];
    docblock?: String;
}

export function getReflectionData(
    classIdOrValue: any,
): JsReflectionData | undefined;
