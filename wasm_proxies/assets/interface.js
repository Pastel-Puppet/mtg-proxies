import init, {generate_proxies_from_textbox, generate_proxies_from_file_contents} from './pkg/wasm_proxies.js';

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
    await generate_proxies_from_textbox(getCustomCards(), document.getElementById("deck-diff").checked)
        .catch((error) => {
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
                        await generate_proxies_from_file_contents(reader.result, file.type, old_reader.result, old_file.type, getCustomCards())
                            .catch((error) => {
                                window.alert(error);
                            })
                            .finally(() => {
                                document.getElementById("loading-overlay").style.display = "none";
                            });
                    }
                    old_reader.readAsText(old_file);
                }
            } else {
                await generate_proxies_from_file_contents(reader.result, file.type, null, null, getCustomCards())
                    .catch((error) => {
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

function clearUploadedCustomCardsClicked() {
    const custom_cards_upload = document.getElementById("custom-cards-upload");
    custom_cards_upload.value = null;
    updateFileSelectionText("custom-cards-upload");
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

    if (file_list.length == 0) {
        text_element.innerText = default_message;
        return;
    }

    let text = "";
    for (const file of file_list) {
        text = [text, file.name].join(" ");
    }

    text_element.innerText = text;
}

function switchDeckControlsTab(selected, other, selected_controls, other_controls) {
    selected.className = selected.className.replace("clickable", "active");
    other.className = other.className.replace("active", "clickable");
    other_controls.style.display = "none";
    selected_controls.style.display = "";
}

toggleDeckDiff();
await init();

document.getElementById("proxies-txt-button").addEventListener("click", proxiesTxtButtonClicked);
document.getElementById("proxies-file-button").addEventListener("click", proxiesFileButtonClicked);

document.getElementById("custom-cards-clear-upload").addEventListener("click", clearUploadedCustomCardsClicked);

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
document.getElementById("custom-cards-upload").addEventListener("change", custom_cards_upload_callback());

proxies_file_select_callback();
old_proxies_file_select_callback();
custom_cards_upload_callback();

document.getElementById("deck-paste-option").addEventListener("click", (event) => switchDeckControlsTab(
    event.target,
    document.getElementById("deck-file-option"),
    document.getElementById("deck-paste-controls"),
    document.getElementById("deck-file-controls")
));
document.getElementById("deck-file-option").addEventListener("click", (event) => switchDeckControlsTab(
    event.target,
    document.getElementById("deck-paste-option"),
    document.getElementById("deck-file-controls"),
    document.getElementById("deck-paste-controls")
));

document.getElementById("loading-overlay").style.display = "none";
