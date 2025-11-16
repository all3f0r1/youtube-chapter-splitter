# Solution pour le cover art sur toutes les pistes

## Problème
Par défaut, ffmpeg ne copie la pochette que sur la première piste lors du découpage d'un album.

## Solution en deux étapes

### Étape 1 : Extraire la pochette du fichier original
```bash
ffmpeg -i "$input" -an -vcodec copy "$cover_extracted"
```

### Étape 2 : Découper par chapitre et ajouter la pochette à chaque piste
```bash
ffmpeg -i "$input" -i "$cover_extracted" \
    -ss "$start" -to "$end" \
    -map 0:a -map 1:v \
    -c:a copy -c:v copy \
    -metadata:s:v title="Album cover" \
    -metadata:s:v comment="Cover (front)" \
    -disposition:v attached_pic \
    "output.m4a"
```

## Alternative simplifiée (sans extraction préalable)
```bash
ffmpeg -i "$input" -i "$input" \
    -ss "$start" -to "$end" \
    -map 0:a -map 1:v \
    -c copy \
    -disposition:v attached_pic \
    "piste_$chapter_num.m4a"
```

Cette approche utilise le même fichier deux fois : une fois pour l'audio et une fois pour extraire la pochette.

## Formats supportés
MP3, M4A, MP4, OGG, WMA et FLAC

## Sources
- https://www.bannerbear.com/blog/how-to-add-a-cover-art-to-audio-files-using-ffmpeg/
- https://www.baeldung.com/linux/terminal-music-add-album-art
- https://gist.github.com/FossPrime/6fcf168cd57cc1d686e4d0a378956d0
