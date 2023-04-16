
export type SpecifierKind = 'None' | 'All' | 'First' | 'Last'
    | { Nth: number }
    | { BackNth: number }
    | { ExplicitNths: number[] }
    | { EveryNth: [number, number] }
    | { ExplicitList: number[] };

export const SPECIFIER_KINDS = [
    'None',
    'All',
    'First',
    'Last',
    'Nth',
    'BackNth',
    'ExplicitNths',
    'EveryNth',
    'ExplicitList'
];

export type Specifier<MIN, MAX> = {
    min: MIN,
    max: MAX,
    kind: SpecifierKind
};

export type Day = { day: number };
export type Month = { month: number };
export type Monday = { day: 0 };
export type Sunday = { day: 6 };
export type January = { month: 0 };
export type December = { month: 11 };

export function isDay(unknownType: object): unknownType is Day {
    return Object.hasOwn(unknownType, 'day');
}

export function isMonth(unknownType: object): unknownType is Month {
    return Object.hasOwn(unknownType, 'month');
}

export type Interval = {
    minutes: Specifier<0, 59>,
    hours: Specifier<0, 23>,
    weekdays: Specifier<Monday, Sunday>,
    monthdays: Specifier<0, 31>,
    weeks: Specifier<0, 52>,
    months: Specifier<January, December>
};

export type ProfileConfig = {
    name: string,
    uuid: string,
    target_dir: string,
    files_to_include: string[],
    dirs_to_include: string[],
    files_to_exclude: string[],
    dirs_to_exclude: string[],
    interval: Interval,
    next_backup: string,
};
