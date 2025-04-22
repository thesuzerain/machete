type ClickOutsideConfig = {
    handler: () => void;
};

export function clickOutside(node: HTMLElement, config: ClickOutsideConfig) {
    const handleClick = (event: MouseEvent) => {
        if (!node.contains(event.target as Node)) {
            config.handler();
        }
    };

    document.addEventListener('click', handleClick, true);

    return {
        destroy() {
            document.removeEventListener('click', handleClick, true);
        }
    };
} 