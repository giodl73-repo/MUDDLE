import {test,expect} from "@playwright/test";
import {createHash} from "node:crypto";
import {execFileSync} from "node:child_process";
test("packaged delivery matches all hashes and exact corresponding-software commit",async({page,request})=>{
 test.setTimeout(180000);await page.goto("./");const proof=await(await request.get("provenance.json")).json(),inventory=await(await request.get("inventory.json")).json();const expected=process.env.MUDDLE_EXPECTED_COMMIT||execFileSync("git",["rev-parse","HEAD"],{encoding:"utf8"}).trim();expect(proof.commit).toBe(expected);expect(inventory.commit).toBe(expected);expect(proof.source_scope).toContain("NOT a full repository");
 for(const record of inventory.files){const response=await request.get(record.path);expect(response.ok(),record.path).toBe(true);const data=await response.body();expect(data.length,record.path).toBe(record.bytes);expect(createHash("sha256").update(data).digest("hex"),record.path).toBe(record.sha256);}expect((await request.get("pkg/muddle_browser_player_bg.wasm")).headers()["content-type"]).toContain("application/wasm");
});
test("source/legal routes remain independent when WASM is unavailable",async({page,request})=>{
 await page.route("**/*.wasm",route=>route.abort());await page.goto("./");await expect(page.locator("#status")).toContainText(/failed|unavailable/i);for(const name of ["Software source ZIP","Compiled WASM","Build instructions","Build provenance","Licenses and notices","Source scope"]){const link=page.getByRole("link",{name,exact:true});await expect(link).toBeVisible();expect((await request.get(await link.getAttribute("href"))).ok()).toBe(true);}
});
