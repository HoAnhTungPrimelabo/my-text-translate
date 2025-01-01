const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const { getCurrentWindow } = window.__TAURI__.window;


listen("tauri://focus", () => {
    $("#users-contain2").show();
});
listen("tauri://blur", () => {
    $("#users-contain2").hide();
});

function setViewSetting(params) {
    $('#number-size').val(params['font-size']);
    $('#range-opacity').val(params['opacity']);
    $('#select-mode').val(params['mode']);
    if (params['always-on-top'] == 1) {
        $('#always-on-top').prop('checked', true);
    } else {
        $('#always-on-top').prop('checked', false);
    }
}
function refreshViewSetting() {
    $(document.body).css("background-color", `rgba(0, 0, 0, ${$('#range-opacity').val()})`);
    $(document.body).css("font-size", $('#number-size').val() + "px");
}
async function loadBackendSetting() {
    const params = await invoke("load_setting");
    setViewSetting(params);
    refreshViewSetting();
}

async function saveSetting() {
    var obj = {};
    obj['font-size'] = $('#number-size').val();
    obj['opacity'] = $('#range-opacity').val();
    obj['always-on-top'] = $('#always-on-top').prop('checked') ? 1 : 0;
    obj['mode'] = $('#select-mode').val();
    await invoke("save_setting", { data: obj});
    loadBackendSetting();
}

function filterAlphabets(word) {
    return word.replace(/[^a-zA-Z]/g, "");
}

var arr_anki = [];

listen("backend_anki", (event) => {
    arr_anki = event.payload
});

listen("backend_text_vn", (event) => {
    //$("#text-vn").text(event.payload);
});
listen("backend_text_en", (event) => {
    $("#text-vn").text(event.payload);
    let words = event.payload.split(" "); // Tách câu thành các từ
    let wrappedWords = words.map(word => {
        let trimword = filterAlphabets(word.toLowerCase());
        if (arr_anki.indexOf(trimword) > -1) {
            return `<span class="text-match1">${word}</span>`
        } else if (trimword.length == 5) {
            let tmp = trimword.slice(0, -2);
            if (arr_anki.filter(s => { return s.indexOf(tmp) == 0;}).length > 0) {
                return `<span class="text-match2">${word}</span>`
            } else {
                return `<span>${word}</span>`
            }
        } else if (trimword.length > 6) {
            let tmp = trimword.slice(0, -3);
            if (arr_anki.filter(s => s.indexOf(tmp) == 0).length > 0) {
                return `<span class="text-match2">${word}</span>`
            } else {
                return `<span>${word}</span>`
            }
        } else {
            return `<span>${word}</span>`
        }
    }); // Bọc mỗi từ trong <span>
    $("#text-en").html(wrappedWords.join(" "));
});

var dialog = $( "#dialog-form" ).dialog({
    autoOpen: false,
    height: 200,
    width: 200,
    modal: true,
  });

// var isDragging = false;
// $("#setting-btn")
// .mousedown(function() {
//     isDragging = false;
// })
// .mousemove(function() {
//     isDragging = true;
// })
// .mouseup(function() {
//     var wasDragging = isDragging;
//     isDragging = false;
//     if (!wasDragging) {
//         dialog.dialog( "open" );
//     }
// });
$( "#setting-btn" ).on( "click", function() {
    dialog.dialog( "open" );
});





$( "#btn-save" ).on( "click", function() {
    saveSetting();
    dialog.dialog( "close" );
});
$( "#btn-cancel" ).on( "click", function() {
    dialog.dialog( "close" );
});

$( document ).on( "click", "span.text-match1", async function() {
    let trimword = filterAlphabets($(this).text());
    await invoke("send_to_anki", { text: trimword });
});

$( document ).on( "click", "span.text-match2", async function() {
    let trimword = filterAlphabets($(this).text());
    if (trimword.length == 5) {
        trimword = trimword.slice(0, -2);
    } else if (trimword.length > 6) {
        trimword = trimword.slice(0, -3);
    }
    await invoke("send_to_anki", { text: trimword });
});

window.addEventListener("DOMContentLoaded", () => {
    loadBackendSetting();
});
