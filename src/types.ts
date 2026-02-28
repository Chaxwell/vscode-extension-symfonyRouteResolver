/** Location of a route controller declaration inside a PHP file. */
export interface RouteInfo {
    /** Absolute path to the PHP file. */
    file: string;
    /** 1-indexed line number of the class or method declaration. */
    line: number;
}

/** A single occurrence of a route name found in a document. */
export interface TextMatch {
    route: string;
    position: {
        start: { line: number; col: number };
        end: { line: number; col: number };
    };
    file: RouteInfo;
}

/** Response sent by the sidecar when an `index` command succeeds. */
export interface IndexedResponse {
    type: 'indexed';
    id: number;
    count: number;
}

/** Response sent by the sidecar when a `search` command succeeds. */
export interface SearchedResponse {
    type: 'searched';
    id: number;
    matches: TextMatch[];
}

/** Error response from the sidecar. */
export interface ErrorResponse {
    type: 'error';
    id: number;
    message: string;
}

export type SidecarResponse = IndexedResponse | SearchedResponse | ErrorResponse;
