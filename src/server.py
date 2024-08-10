from ArticleExtractor import ArticleExtractor
from flask import Flask, render_template, request, redirect, url_for

app = Flask(__name__)

@app.route('/', methods=['GET', 'POST'])
def index():
    articles = ["AI Act", "DGA", "GDPR"]
    
    if request.method == 'POST':
        source = request.form.get('source')
        destination = request.form.get('destination')
        article_extractor = ArticleExtractor({"extraction_method": "external"})
        article_extractor.run(source_doc=source, target_doc=destination)
        return render_template('legal_document_relations.html')
    
    return render_template('index.html', articles=articles)

if __name__ == '__main__':
    app.run(debug=False)
