Array.from(document.getElementsByTagName('input')).map(i => { if (i.type == "checkbox") { i.disabled = false } });
