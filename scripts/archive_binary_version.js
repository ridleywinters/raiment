const glob = require('glob');
const sh = require('shelljs');
const _ = require('lodash');

let files = glob.sync('*');
files = _.filter(files, (file) => sh.test('-d', file));
files = _.filter(files, (file) => file.match(/voxel-([0-9]+)$/));
files = files.sort().reverse();
console.log(files)

let versionString = '00';
if (files.length > 0) {
    let latest = files[0];
    let m = latest.match(/voxel-([0-9]+)$/);
    if (m) {
        let version = parseInt(m[1]) + 1;
        versionString = `${version}`.padStart(2, '0');
    } else {
        versionString = `${version}`.padStart(2, '0');
    }
}


const src = 'voxel-main';
const dst = `voxel-${versionString}`;
console.log('-R', src, dst);
console.log(`main.rs archiving as version ${versionString}`);
sh.cp('-R', src, dst);

