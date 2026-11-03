getHost = (a, b, c, d) => [
    "https://",
    ["reading", "api", "api3", "api5", "novel", ""][(a == 4 ? 5 : b) || 0],
    ["", "-normal"][c || 0],
    ["", "-hl", "-lf", "-lq", "-sinfonlinea", "-sinfonlineb", "-sinfonlinec"][d || 0],
    [".", ""][a == 4 ? 1 : 0],
    ["snssdk", "fqnovel", "fanqiesdk", "toutiaoapi", "fanqienovel"][a || 0],
    ".com",
].join("");

javaImport = new JavaImporter();
javaImport.importPackage(
    Packages.okhttp3,
    Packages.cn.hutool.core.util,
    Packages.cn.hutool.core.codec,
    Packages.cn.hutool.crypto.digest,
);

with (javaImport) {
    function post(url, body, map) {
        const request = new Request.Builder()
            .url(url)
            .post(RequestBody.create(body, MediaType.parse("application/json")));
        if (map) {
            for (let name in map) request.addHeader(name, map[name]);
        }
        return JSON.parse(new OkHttpClient().newCall(request.build()).execute().body().string());
    }

    function xGorgon(path, params, data, ck) {
        const { java, source } = this;
        params = [
            params,
            "aid=1967",
            "channel=0",
            "os_version=0",
            "app_name=novelapp",
            "version_code=58932",
            "device_platform=android",
            "device_type=unknown",
        ].join("&").split("&").sort().join("&").replace(/^&+/, "");
        if (!data) {
            path = "/reading/bookapi/" + path + "/v/?";
        }
        const url = getHost() + path + params;
        for (let item of (source.getLoginHeader() || "").split("&")) {
            if (item.startsWith("device_type")) {
                // Keep the same parsing branch as the upstream source.
            }
        }

        const md5 = (str) => DigestUtil.md5Hex(str);
        const rStr = (str) => StrUtil.reverse(str);
        const Hex = (num) => num.toString(16).padStart(2, "0");
        const rHex = (num) => parseInt(rStr(Hex(num)), 16);

        function rBin(num) {
            const bin = num.toString(2).padStart(8, "0");
            return parseInt(rStr(bin), 2);
        }

        function getHex(cookie) {
            let hex = md5(params);
            hex += data ? md5(data) : "0".repeat(8);
            hex += cookie ? md5(cookie) : "0".repeat(8);
            return hex;
        }

        function calculate(hex, cookie) {
            const len = 0x14;
            const key = [0xDF, 0x77, 0xB9, 0x40, 0xB9, 0x9B, 0x84, 0x83, 0xD1, 0xB9, 0xCB, 0xD1, 0xF7, 0xC2, 0xB9, 0x85, 0xC3, 0xD0, 0xFB, 0xC3];
            const paramList = [];
            for (let i = 0; i < 9; i += 4) {
                const temp = hex.substring(8 * i, 8 * (i + 1));
                for (let j = 0; j < 4; j++) {
                    paramList.push(parseInt(temp.substring(j * 2, (j + 1) * 2), 16));
                }
            }
            paramList.push(0x0, 0x6, 0xB, 0x1C);
            const T = Math.floor(Date.now() / 1000);
            paramList.push((T >> 24) & 0xFF, (T >> 16) & 0xFF, (T >> 8) & 0xFF, T & 0xFF);
            const eorResultList = [];
            for (let i = 0; i < paramList.length; i++) {
                eorResultList.push(paramList[i] ^ key[i % len]);
            }
            for (let A, B, C, D, i = 0; i < len; i++) {
                A = rHex(eorResultList[i]);
                B = eorResultList[(i + 1) % len];
                C = rBin(A ^ B);
                D = ((C ^ 0xFFFFFFFF) ^ len) & 0xFF;
                eorResultList[i] = D;
            }
            let result = "";
            for (const param of eorResultList) result += Hex(param);
            const option = {
                headers: {
                    "X-Khronos": String(T),
                    "X-Gorgon": "0404b0d30000" + result,
                    "User-Agent": "com.dragon.read",
                    "Cookie": cookie ? cookie : "",
                },
            };
            option.body = String(data ? data : "");
            if (data) return post(url, data, option.headers);
            return url + "," + JSON.stringify(option);
        }

        return calculate(getHex(ck), ck);
    }
}
