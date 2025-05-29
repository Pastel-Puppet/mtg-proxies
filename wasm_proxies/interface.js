import init, {generate_proxies_from_textbox, generate_proxies_from_file_contents} from './pkg/wasm_proxies.js';

let imageBlobUrls = [];

async function run() {
    await init();
}

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
    await generate_proxies_from_textbox(getCustomCards(), document.getElementById("deck-diff").checked);
}

function proxiesFileButtonClicked() {
    const file = document.getElementById("proxies-file-select").files[0];

    if (file) {
        const reader = new FileReader();
        reader.onload = async () => {
            if (document.getElementById("deck-diff").checked) {
                const old_file = document.getElementById("old-proxies-file-select").files[0];

                if (old_file) {
                    const old_reader = new FileReader();
                    old_reader.onload = async () => {
                        await generate_proxies_from_file_contents(reader.result, file.type, old_reader.result, old_file.type, getCustomCards());
                    }
                    old_reader.readAsText(old_file);
                }
            } else {
                await generate_proxies_from_file_contents(reader.result, file.type, null, null, getCustomCards());
            }
        };
        reader.readAsText(file);
    }
}

function clearUploadedCustomCardsClicked() {
    const custom_cards_upload = document.getElementById("custom-cards-upload");
    custom_cards_upload.value = null;
}

function toggleDeckDiff() {
    if (document.getElementById("deck-diff").checked) {
        document.getElementById("old-deck-list").style.display = "inline-block";
        document.getElementById("old-proxies-file-select").style.display = "inline-block";
    } else {
        document.getElementById("old-deck-list").style.display = "none";
        document.getElementById("old-proxies-file-select").style.display = "none";
    }
}

toggleDeckDiff();
run();

document.getElementById("proxies-txt-button").addEventListener("click", proxiesTxtButtonClicked);
document.getElementById("proxies-file-button").addEventListener("click", proxiesFileButtonClicked);
document.getElementById("custom-cards-clear-upload").addEventListener("click", clearUploadedCustomCardsClicked);
document.getElementById("deck-diff").addEventListener("change", toggleDeckDiff);
