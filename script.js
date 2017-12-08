'use strict;'

window.onload = function () {
    window.exports = {};

    let serviceName = document.querySelector('#serviceName'),
        keyword = document.querySelector('#keyword'),
        passwordLength = document.querySelector('#passwordLength'),
        useSpecialCharacters = document.querySelector('#useSpecialCharacters'),
        password = document.querySelector('#password'),
        copy = document.querySelector('#copy');

    let imports = {
        round: Math.round
    };

    fetchAndInstantiate('build/main.wasm', { env: imports })
        .then(mod => {
            exports = mod.exports;

            let generate_password = () => {
                if (passwordLength.value >= 8 && passwordLength.value <= 60) {
                    let s = exports.generate_password_c(
                        newString(exports, serviceName.value),
                        newString(exports, keyword.value),
                        passwordLength.value,
                        useSpecialCharacters.checked
                    );
                    password.value = copyCStr(exports, s);
                } else password.value = '';
            };

            serviceName.oninput = generate_password;
            keyword.oninput = generate_password;
            passwordLength.oninput = generate_password;
            useSpecialCharacters.onchange = generate_password;
        });

    copy.onclick = () => {
        password.select();
        document.execCommand("Copy");
    };
};
