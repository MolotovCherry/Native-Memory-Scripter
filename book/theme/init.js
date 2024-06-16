(() => {
    const themeList = {
        'light': 'light',
        'rust': 'light',
        'coal': 'dark',
        'navy': 'dark',
        'ayu': 'dark',
        'latte': 'light',
        'frappe': 'dark',
        'macchiato': 'dark',
        'mocha': 'dark'
    };

    const updateImages = (theme) => {
        const mTheme = themeList[theme] || 'dark';

        const elems = document.querySelectorAll("img");
        for (const elem of elems) {
            const src = elem.dataset.rsrc;
            if (src) {
                elem.src = src.replace("$theme", mTheme);
            }
        }
    };

    const saveSrc = () => {
        const elems = document.querySelectorAll("img");
        for (const elem of elems) {
            elem.setAttribute("data-src", elem.src);
        }
    };

    const html = document.documentElement;
    let currentClass = html.className;

    const attrObserver = new MutationObserver((mutations) => {
        mutations.forEach(mu => {
            if (mu.type !== "attributes" && mu.attributeName !== "class") { return; }

            const newClass = html.className;
            if (newClass !== currentClass) {
                updateImages(newClass);
            }

            currentClass = html.className;
        });
    });

    attrObserver.observe(html, { attributes: true });

    saveSrc();
    updateImages(currentClass);
})();
