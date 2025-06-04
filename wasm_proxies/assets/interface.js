import init, {generate_proxies_from_textbox, generate_proxies_from_file_contents, get_printings_for_card} from './pkg/wasm_proxies.js';

let imageBlobUrls = [];

function getCustomCards() {
    for (const blobUrl of imageBlobUrls) {
        URL.revokeObjectURL(blobUrl);
    }

    imageBlobUrls = [];

    for (const file of document.getElementById("custom-cards-upload").files) {
        imageBlobUrls.push(URL.createObjectURL(file));
    }

    return imageBlobUrls;
}

async function proxiesTxtButtonClicked() {
    document.getElementById("loading-overlay").style.display = "block";
    await generate_proxies_from_textbox(getCustomCards(), document.getElementById("deck-diff").checked, cardClickedWrapper)
        .then(() => {
            updatePrintButton();
        })
        .catch((error) => {
            console.error(error);
            window.alert(error);
        })
        .finally(() => {
            document.getElementById("loading-overlay").style.display = "none";
        });
}

function proxiesFileButtonClicked() {
    const file = document.getElementById("proxies-file-select").files[0];

    if (file) {
        document.getElementById("loading-overlay").style.display = "block";
        const reader = new FileReader();
        reader.onload = async () => {
            if (document.getElementById("deck-diff").checked) {
                const old_file = document.getElementById("old-proxies-file-select").files[0];

                if (old_file) {
                    const old_reader = new FileReader();
                    old_reader.onload = async () => {
                        await generate_proxies_from_file_contents(reader.result, file.type, old_reader.result, old_file.type, getCustomCards(), cardClickedWrapper)
                            .then(() => {
                                updatePrintButton();
                            })
                            .catch((error) => {
                                console.error(error);
                                window.alert(error);
                            })
                            .finally(() => {
                                document.getElementById("loading-overlay").style.display = "none";
                            });
                    }
                    old_reader.readAsText(old_file);
                }
            } else {
                await generate_proxies_from_file_contents(reader.result, file.type, null, null, getCustomCards(), cardClickedWrapper)
                    .then(() => {
                        updatePrintButton();
                    })
                    .catch((error) => {
                        console.error(error);
                        window.alert(error);
                    })
                    .finally(() => {
                        document.getElementById("loading-overlay").style.display = "none";
                    });
            }
        };
        reader.onerror = async () => {
            if (reader.error) {
                window.alert(reader.error);
                document.getElementById("loading-overlay").style.display = "none";
            }
        };
        reader.onabort = async () => {
            console.log("File read aborted");
            document.getElementById("loading-overlay").style.display = "none";
        }
        reader.readAsText(file);
    }
}

async function cardClickedWrapper(card_clicked_data) {
    await cardClicked(
        card_clicked_data.card_face_images_array,
        card_clicked_data.prints_search_url,
        card_clicked_data.card_name,
        card_clicked_data.is_custom_card
    );
}

async function cardClicked(image_urls, prints_search_uri, card_name, is_custom_card) {
    const card_printings_overlay = document.getElementById("card-overlay");
    card_printings_overlay.textContent = "";

    // Add printing data for current printing only while the rest load.
    let card_printing_node = document.createElement("span");
    card_printing_node.className = "card-printing";

    let card_wrapper_node = document.createElement("span");
    card_wrapper_node.className = "selected-card-wrapper";

    if (!is_custom_card) {
        let left_button_node = document.createElement("button");
        left_button_node.className = "option-button clickable";
        left_button_node.innerText = "←";
        left_button_node.disabled = "true";
        card_wrapper_node.appendChild(left_button_node);
    }

    for (const card_face of image_urls) {
        let card_node = document.createElement("img");
        card_node.src = card_face;
        card_node.className = "selected-card";
        card_wrapper_node.appendChild(card_node)
    }

    if (!is_custom_card) {
        let right_button_node = document.createElement("button");
        right_button_node.className = "option-button clickable";
        right_button_node.innerText = "→";
        right_button_node.disabled = "true";
        card_wrapper_node.appendChild(right_button_node);
    }
    
    card_printing_node.appendChild(card_wrapper_node);

    if (is_custom_card) {
        let card_printing_index_node = document.createElement("p");
        card_printing_index_node.className = "card-printing-index boxed";
        card_printing_index_node.innerText = "This is a custom card\n\nNo printing information is available";
        card_printing_node.appendChild(card_printing_index_node);
    } else {
        let card_printing_index_node = document.createElement("p");
        card_printing_index_node.className = "card-printing-index boxed";
        card_printing_index_node.innerText = "?/?\n\nLoading printings for " + card_name + "...";
        card_printing_node.appendChild(card_printing_index_node);
    }

    card_printings_overlay.appendChild(card_printing_node);
    card_printings_overlay.style.display = "";

    if (is_custom_card) {
        return;
    }

    // Fetch printing data.
    let printings = await get_printings_for_card(prints_search_uri, image_urls[0], card_name)
        .catch((error) => {
            console.error(error);
            window.alert(error);
        });

    // Add card printings to overlay.
    let previous_printing = null;
    let previous_right_button = null;
    let first_printing = null;
    let first_left_button = null;
    let printing_nodes = [];

    for (const [index, printing] of printings.printings.entries()) {
        let current_previous_printing = previous_printing;
        let current_previous_right_button = previous_right_button;

        let card_printing_node = document.createElement("span");
        card_printing_node.className = "card-printing";

        let current_printing_faces_right = printing.faces;
        if (current_previous_right_button != null) {
            current_previous_right_button.onclick = () => {
                if (current_previous_printing != null) {
                    current_previous_printing.style.display = "none";
                }
                changePrinting(printings.printings[(index - 1) % printings.printings.length].faces, current_printing_faces_right, prints_search_uri, card_name);
                card_printing_node.style.display = "";
            };
        }

        let card_wrapper_node = document.createElement("span");
        card_wrapper_node.className = "selected-card-wrapper";

        let left_button_node = document.createElement("button");
        left_button_node.className = "option-button clickable";
        left_button_node.innerText = "←";

        let current_printing_faces_left = printing.faces;
        if (current_previous_printing != null) {
            left_button_node.onclick = () => {
                card_printing_node.style.display = "none";
                changePrinting(current_printing_faces_left, printings.printings[(index - 1) % printings.printings.length].faces, prints_search_uri, card_name);
                current_previous_printing.style.display = "";
            };
        }
        card_wrapper_node.appendChild(left_button_node);

        for (const card_face of printing.faces) {
            let card_node = document.createElement("img");
            card_node.src = card_face;
            card_node.className = "selected-card";
            if (index != printings.current_index) {
                card_node.loading = "lazy";
            }
            card_node.fetchPriority = "high";
            card_node.dataset.index = index;
            card_node.dataset.set = printing.set;
            card_node.dataset.collectorNumber = printing.collector_number;
            card_wrapper_node.appendChild(card_node)
        }

        let right_button_node = document.createElement("button");
        right_button_node.className = "option-button clickable";
        right_button_node.innerText = "→";
        card_wrapper_node.appendChild(right_button_node);

        card_printing_node.appendChild(card_wrapper_node);

        let card_printing_index_node = document.createElement("p");
        card_printing_index_node.className = "card-printing-index boxed";
        card_printing_index_node.innerText = (index + 1) + "/" + printings.printings.length + "\n" + printing.set + " - " + printing.collector_number + "\n";
        
        let card_link_node = document.createElement("a");
        card_link_node.href = printing.scryfall_url;
        card_link_node.innerText = "View more information about " + card_name + " on Scryfall";
        card_printing_index_node.appendChild(card_link_node)

        card_printing_node.appendChild(card_printing_index_node);

        if (index != printings.current_index) {
            card_printing_node.style.display = "none";
        }

        printing_nodes.push(card_printing_node);

        previous_printing = card_printing_node;
        previous_right_button = right_button_node;
        if (first_printing === null) {
            first_printing = card_printing_node;
        }
        if (first_left_button === null) {
            first_left_button = left_button_node;
        }
    }

    if (first_left_button != null) {
        first_left_button.onclick = () => {
            if (first_printing != null) {
                first_printing.style.display = "none";
            }
            if (previous_printing != null) {
                previous_printing.style.display = "";
            }
            changePrinting(printings.printings[0].faces, printings.printings[printings.printings.length - 1].faces, prints_search_uri, card_name);
        }
    }

    if (previous_right_button != null) {
        previous_right_button.onclick = () => {
            if (previous_printing != null) {
                previous_printing.style.display = "none";
            }
            if (first_printing != null) {
                first_printing.style.display = "";
            }
            changePrinting(printings.printings[printings.printings.length - 1].faces, printings.printings[0].faces, prints_search_uri, card_name);
        }
    }

    card_printings_overlay.replaceChildren(...printing_nodes);
}

function changePrinting(old_printing_urls, new_printing_urls, prints_search_uri, card_name) {
    for (const [old_printing, new_printing] of old_printing_urls.map((new_url, index) => [new_url, new_printing_urls[index]])) {
        console.log("Changing " + old_printing + " to " + new_printing);
        let cards = document.getElementById("proxies").children;
        for (const card of cards) {
            if (card.className === "card-face" && card.src === old_printing) {
                card.src = new_printing;
                card.onclick = cardClicked.bind(card, new_printing_urls, prints_search_uri, card_name, false);
            }
        }
    }
}

function clearUploadedCustomCardsClicked(update_file_selection_text_callback) {
    const custom_cards_upload = document.getElementById("custom-cards-upload");
    custom_cards_upload.value = null;
    update_file_selection_text_callback();
}

function toggleDeckDiff() {
    if (document.getElementById("deck-diff").checked) {
        document.getElementById("old-deck-list").style.display = "";
        document.getElementById("old-proxies-file-select-wrapper").style.display = "";
    } else {
        document.getElementById("old-deck-list").style.display = "none";
        document.getElementById("old-proxies-file-select-wrapper").style.display = "none";
    }
}

function updateFileSelectionText(file_element, text_element, default_message) {
    const file_list = file_element.files;

    if (file_list.length === 0) {
        text_element.innerText = default_message;
        return;
    }

    let text = "";
    for (const file of file_list) {
        text = [text, file.name].join(" ");
    }

    text_element.innerText = text;
}

function switchTab(selected, other, selected_controls, other_controls) {
    selected.className = selected.className.replace("clickable", "active");
    other.className = other.className.replace("active", "clickable");
    other_controls.style.display = "none";
    selected_controls.style.display = "";
}

function updatePrintButton() {
    if (document.getElementById("proxies").hasChildNodes()) {
        document.getElementById("proxies-txt-print-button").disabled = false;
        document.getElementById("proxies-file-print-button").disabled = false;
    } else {
        document.getElementById("proxies-txt-print-button").disabled = true;
        document.getElementById("proxies-file-print-button").disabled = true;
    }
}

toggleDeckDiff();
await init();

document.getElementById("proxies-txt-button").addEventListener("click", proxiesTxtButtonClicked);
document.getElementById("proxies-file-button").addEventListener("click", proxiesFileButtonClicked);

document.getElementById("proxies-txt-print-button").addEventListener("click", () => window.print());
document.getElementById("proxies-file-print-button").addEventListener("click", () => window.print());

document.getElementById("deck-diff").addEventListener("change", toggleDeckDiff);

document.getElementById("proxies-file-select-wrapper").addEventListener("click", () => document.getElementById("proxies-file-select").click());
document.getElementById("old-proxies-file-select-wrapper").addEventListener("click", () => document.getElementById("old-proxies-file-select").click());
document.getElementById("custom-cards-upload-wrapper").addEventListener("click", () => document.getElementById("custom-cards-upload").click());

const proxies_file_select_callback = () => updateFileSelectionText(
    document.getElementById("proxies-file-select"),
    document.getElementById("proxies-file-select-text"),
    "Select deck file"
);
document.getElementById("proxies-file-select").addEventListener("change", proxies_file_select_callback);

const old_proxies_file_select_callback = () => updateFileSelectionText(
    document.getElementById("old-proxies-file-select"),
    document.getElementById("old-proxies-file-select-text"),
    "Select previous deck file"
);
document.getElementById("old-proxies-file-select").addEventListener("change", old_proxies_file_select_callback);

const custom_cards_upload_callback = () => updateFileSelectionText(
    document.getElementById("custom-cards-upload"),
    document.getElementById("custom-cards-upload-text"),
    "No custom cards selected"
);
document.getElementById("custom-cards-upload").addEventListener("change", custom_cards_upload_callback);

document.getElementById("custom-cards-clear-upload").addEventListener("click", () => clearUploadedCustomCardsClicked(custom_cards_upload_callback));

proxies_file_select_callback();
old_proxies_file_select_callback();
custom_cards_upload_callback();

document.getElementById("deck-paste-option").addEventListener("click", (event) => switchTab(
    event.target,
    document.getElementById("deck-file-option"),
    document.getElementById("deck-paste-controls"),
    document.getElementById("deck-file-controls")
));
document.getElementById("deck-file-option").addEventListener("click", (event) => switchTab(
    event.target,
    document.getElementById("deck-paste-option"),
    document.getElementById("deck-file-controls"),
    document.getElementById("deck-paste-controls")
));

document.getElementById("usage-help-option").addEventListener("click", (event) => switchTab(
    event.target,
    document.getElementById("format-help-option"),
    document.getElementById("usage-help"),
    document.getElementById("format-help")
));
document.getElementById("format-help-option").addEventListener("click", (event) => switchTab(
    event.target,
    document.getElementById("usage-help-option"),
    document.getElementById("format-help"),
    document.getElementById("usage-help")
));

document.getElementById("supported-formats-button").addEventListener("click", () => {
    document.getElementById("help-overlay").style.display = "";
});

document.getElementById("card-overlay").addEventListener("click", (event) => {
    if (event.target.tagName.toUpperCase() === "SPAN" || event.target.tagName.toUpperCase() === "DIV") {
        event.currentTarget.style.display = "none";
    }
});

document.getElementById("help-overlay").addEventListener("click", (event) => {
    if (event.target.id === "help-overlay") {
        event.target.style.display = "none";
    }
});

document.body.onkeydown = (event) => {
    if (event.key === "Escape") {
        document.getElementById("card-overlay").style.display = "none";
        document.getElementById("help-overlay").style.display = "none";
    }
}

document.getElementById("loading-overlay").style.display = "none";
