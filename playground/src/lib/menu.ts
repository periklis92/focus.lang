export type MenuItem = {
    title: string,
    id: string,
    markdown?: string,
    code?: string,
}

export type Menu = {
    items: MenuItem[],
    title: string,
    id?: string,
    class?: string,
    icon?: string
}