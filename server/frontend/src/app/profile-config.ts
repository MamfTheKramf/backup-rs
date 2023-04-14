
export type SpecifierKind = 'None' | 'All' | 'First' | 'Last'
    | { Nth: number }
    | { BackNth: number }
    | { ExplicitNths: number[] }
    | { EveryNth: [number, number] }
    | { ExplicitList: number[] };

export type Specifier<MIN, MAX> = {
    min: MIN,
    max: MAX,
    kind: SpecifierKind
};

export type Monday = { day: 0 };
export type Sunday = { day: 6 };
export type January = { month: 0 };
export type December = { month: 11 };

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
